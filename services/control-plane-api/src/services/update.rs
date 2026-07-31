use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::{UpdateCheckRequest, UpdateCheckResponse, PublishUpdateRequest};
use ed25519_dalek::Signer;

pub async fn check_for_update(
    pool: &PgPool,
    request: UpdateCheckRequest,
) -> Result<UpdateCheckResponse, ApiError> {
    // Get latest version for the OS and channel
    let version = sqlx::query_as::<_, crate::models::AppVersion>(
        "SELECT * FROM app_versions WHERE os = $1 AND channel = $2 AND published_at IS NOT NULL ORDER BY published_at DESC LIMIT 1"
    )
    .bind(&request.os)
    .bind(&request.channel)
    .fetch_optional(pool)
    .await?;

    match version {
        Some(v) => {
            // Check if update is available (version comparison)
            let current_version = parse_version(&request.current_version);
            let latest_version = parse_version(&v.version);

            let update_available = latest_version > current_version;

            Ok(UpdateCheckResponse {
                update_available,
                version: if update_available { Some(v.version) } else { None },
                sha256: if update_available { Some(v.sha256) } else { None },
                signature: if update_available { Some(v.signature) } else { None },
                download_url: if update_available { Some(v.download_url) } else { None },
                critical: v.critical,
                release_notes: if update_available { v.release_notes } else { None },
            })
        }
        None => Ok(UpdateCheckResponse {
            update_available: false,
            version: None,
            sha256: None,
            signature: None,
            download_url: None,
            critical: false,
            release_notes: None,
        }),
    }
}

pub async fn publish_update_metadata(
    pool: &PgPool,
    request: PublishUpdateRequest,
) -> Result<serde_json::Value, ApiError> {
    // Get active update signing key
    let signing_key = sqlx::query_as::<_, crate::models::LicenseSigningKey>(
        "SELECT * FROM license_signing_keys WHERE status = 'active' LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;

    let signing_key = match signing_key {
        Some(k) => k,
        None => return Err(ApiError::InternalError("No active signing key found".to_string())),
    };

    // Load signing key from file
    let key_path = std::env::var("UPDATE_SIGNING_KEY_PATH")
        .unwrap_or_else(|_| "./keys/update_private.pem".to_string());

    let key_bytes = std::fs::read(&key_path)
        .map_err(|e| ApiError::SigningKeyError(format!("Failed to read signing key: {}", e)))?;

    let key_str = String::from_utf8(key_bytes)
        .map_err(|e| ApiError::SigningKeyError(format!("Invalid key file encoding: {}", e)))?;

    // Parse PEM and extract Ed25519 private key
    let key_b64 = key_str
        .replace("-----BEGIN PRIVATE KEY-----", "")
        .replace("-----END PRIVATE KEY-----", "")
        .replace("\n", "")
        .replace("\r", "");

    let key_der = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &key_b64,
    )
    .map_err(|e| ApiError::SigningKeyError(format!("Failed to decode key: {}", e)))?;

    // Extract Ed25519 seed from PKCS8 DER
    if key_der.len() < 48 {
        return Err(ApiError::SigningKeyError("Invalid key format".to_string()));
    }

    let seed: [u8; 32] = key_der[16..48].try_into()
        .map_err(|_| ApiError::SigningKeyError("Failed to extract seed".to_string()))?;

    let signing_key_obj = ed25519_dalek::SigningKey::from_bytes(&seed);

    // Create signature payload
    let payload = serde_json::json!({
        "version": request.version,
        "channel": request.channel,
        "os": request.os,
        "sha256": request.sha256,
        "download_url": request.download_url,
    });

    let payload_bytes = serde_json::to_vec(&payload)
        .map_err(|e| ApiError::SigningKeyError(format!("Failed to serialize payload: {}", e)))?;

    let signature = signing_key_obj.sign(&payload_bytes);

    // Store version in database
    let version_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO app_versions (id, version, channel, os, min_supported_version, sha256, signature, signing_key_id, download_url, release_notes, critical) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
    )
    .bind(version_id)
    .bind(&request.version)
    .bind(&request.channel)
    .bind(&request.os)
    .bind(&request.min_supported_version)
    .bind(&request.sha256)
    .bind(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signature.to_bytes()))
    .bind(signing_key.id)
    .bind(&request.download_url)
    .bind(&request.release_notes)
    .bind(request.critical)
    .execute(pool)
    .await?;

    // Create admin audit log
    let audit_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO admin_audit_logs (id, action, resource_type, resource_id, new_value) VALUES ($1, 'publish_update', 'app_version', $2, $3)"
    )
    .bind(audit_id)
    .bind(version_id.to_string())
    .bind(serde_json::json!({
        "version": request.version,
        "channel": request.channel,
        "os": request.os,
        "critical": request.critical,
    }))
    .execute(pool)
    .await?;

    tracing::info!("Update {} published for {} ({})", request.version, request.os, request.channel);

    Ok(serde_json::json!({
        "success": true,
        "version_id": version_id,
        "version": request.version,
        "server_time": chrono::Utc::now().to_rfc3339(),
    }))
}

fn parse_version(version: &str) -> Vec<i32> {
    version
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect()
}
