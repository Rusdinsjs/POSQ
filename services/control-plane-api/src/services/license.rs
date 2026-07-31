use sqlx::PgPool;
use uuid::Uuid;
use ed25519_dalek::{Signer, SigningKey};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::fs;

use crate::error::ApiError;
use crate::models::LicenseSigningKey;
use crate::models::RefreshLicenseRequest;

pub async fn issue_license_token(
    device_id: &Uuid,
    merchant_id: &Uuid,
    install_id_hash: &str,
    device_public_key_thumbprint: &str,
    signing_key: &LicenseSigningKey,
    runtime_mode: &str,
    grace_until_ts: i64,
) -> Result<String, ApiError> {
    // Load signing key from file
    let key_path = std::env::var("LICENSE_SIGNING_KEY_PATH")
        .unwrap_or_else(|_| "./keys/license_private.pem".to_string());

    let key_bytes = fs::read(&key_path)
        .map_err(|e| ApiError::SigningKeyError(format!("Failed to read signing key: {}", e)))?;

    // Parse PEM and extract Ed25519 private key
    let key_str = String::from_utf8(key_bytes)
        .map_err(|e| ApiError::SigningKeyError(format!("Invalid key file encoding: {}", e)))?;

    // Remove PEM headers and newlines
    let key_b64 = key_str
        .replace("-----BEGIN PRIVATE KEY-----", "")
        .replace("-----END PRIVATE KEY-----", "")
        .replace("\n", "")
        .replace("\r", "");

    let key_der = BASE64.decode(&key_b64)
        .map_err(|e| ApiError::SigningKeyError(format!("Failed to decode key: {}", e)))?;

    // Extract Ed25519 seed from PKCS8 DER
    // Ed25519 PKCS8 private key is 48 bytes: 16 bytes header + 32 bytes seed
    if key_der.len() < 48 {
        return Err(ApiError::SigningKeyError("Invalid key format".to_string()));
    }

    let seed: [u8; 32] = key_der[16..48].try_into()
        .map_err(|_| ApiError::SigningKeyError("Failed to extract seed".to_string()))?;

    let signing_key_obj = SigningKey::from_bytes(&seed);

    // Create token payload
    let now = chrono::Utc::now().timestamp() as usize;
    let exp = now + (30 * 24 * 3600); // 30 days
    let iat = now;

    let payload = serde_json::json!({
        "iss": "posq-server",
        "sub": device_id.to_string(),
        "aud": "posq-desktop",
        "merchant_id": merchant_id.to_string(),
        "device_id": device_id.to_string(),
        "install_id_hash": install_id_hash,
        "device_public_key_thumbprint": device_public_key_thumbprint,
        "token_version": 1,
        "runtime_mode": runtime_mode,
        "key_id": signing_key.key_id,
        "iat": iat,
        "nbf": iat,
        "exp": exp,
        "grace_until": grace_until_ts,
    });

    // Sign the payload
    let payload_bytes = serde_json::to_vec(&payload)
        .map_err(|e| ApiError::SigningKeyError(format!("Failed to serialize payload: {}", e)))?;

    let signature = signing_key_obj.sign(&payload_bytes);

    // Create JWT-like token (header.payload.signature)
    let header = serde_json::json!({
        "alg": "EdDSA",
        "typ": "JWT",
        "kid": signing_key.key_id,
    });

    let header_b64 = BASE64.encode(serde_json::to_vec(&header).unwrap());
    let payload_b64 = BASE64.encode(&payload_bytes);
    let signature_b64 = BASE64.encode(signature.to_bytes());

    let token = format!("{}.{}.{}", header_b64, payload_b64, signature_b64);

    Ok(token)
}

pub async fn refresh_license_token(
    pool: &PgPool,
    request: RefreshLicenseRequest,
) -> Result<crate::models::RefreshLicenseResponse, ApiError> {
    // Parse device ID
    let device_id: Uuid = request.device_id.parse()
        .map_err(|_| ApiError::BadRequest("Invalid device ID".to_string()))?;

    // Get device
    let device = sqlx::query_as::<_, crate::models::Device>(
        "SELECT * FROM devices WHERE id = $1 AND deleted_at IS NULL"
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await?;

    let device = match device {
        Some(d) => d,
        None => return Err(ApiError::NotFound("Device not found".to_string())),
    };

    if device.status != "active" {
        return Err(ApiError::Forbidden("Device is not active".to_string()));
    }

    // Get current license
    let current_license = sqlx::query_as::<_, crate::models::DeviceLicense>(
        "SELECT * FROM device_licenses WHERE device_id = $1 AND revoked_at IS NULL ORDER BY token_version DESC LIMIT 1"
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await?;

    let current_license = match current_license {
        Some(l) => l,
        None => return Err(ApiError::NotFound("No active license found".to_string())),
    };

    // Check if license is expired
    if current_license.valid_until < chrono::Utc::now() {
        return Err(ApiError::Forbidden("License has expired".to_string()));
    }

    // Get signing key
    let signing_key = sqlx::query_as::<_, crate::models::LicenseSigningKey>(
        "SELECT * FROM license_signing_keys WHERE id = $1"
    )
    .bind(current_license.signing_key_id)
    .fetch_optional(pool)
    .await?;

    let signing_key = match signing_key {
        Some(k) => k,
        None => return Err(ApiError::InternalError("Signing key not found".to_string())),
    };

    // Get subscription status
    let subscription = sqlx::query_as::<_, crate::models::Subscription>(
        "SELECT * FROM subscriptions WHERE merchant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC LIMIT 1"
    )
    .bind(device.merchant_id)
    .fetch_optional(pool)
    .await?;

    let runtime_mode = match subscription {
        Some(sub) => {
            if sub.status == "active" && sub.valid_until > chrono::Utc::now() {
                "active"
            } else if sub.grace_until.map(|g| g > chrono::Utc::now()).unwrap_or(false) {
                "grace"
            } else {
                "restricted_expired"
            }
        }
        None => "restricted_expired",
    };

    // Issue new token
    let new_token = issue_license_token(
        &device_id,
        &device.merchant_id,
        &device.install_id_hash,
        &device.device_public_key_thumbprint.unwrap_or_default(),
        &signing_key,
        runtime_mode,
        (chrono::Utc::now() + chrono::Duration::days(37)).timestamp(),
    ).await?;

    // Store new license
    let new_version = current_license.token_version + 1;
    let token_hash = {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(new_token.as_bytes());
        format!("{:x}", hasher.finalize())
    };
    let valid_until = chrono::Utc::now() + chrono::Duration::days(30);
    let grace_until = chrono::Utc::now() + chrono::Duration::days(37);

    sqlx::query(
        "INSERT INTO device_licenses (device_id, token_version, token_hash, signing_key_id, runtime_mode, valid_until, grace_until) VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(device_id)
    .bind(new_version)
    .bind(&token_hash)
    .bind(signing_key.id)
    .bind(runtime_mode)
    .bind(valid_until)
    .bind(grace_until)
    .execute(pool)
    .await?;

    tracing::info!("License refreshed for device {}", device_id);

    Ok(crate::models::RefreshLicenseResponse {
        license_token: new_token,
        server_time: chrono::Utc::now().to_rfc3339(),
        runtime_mode: runtime_mode.to_string(),
    })
}
