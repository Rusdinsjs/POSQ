use sqlx::PgPool;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use rand::Rng;

use crate::error::ApiError;
use crate::models::{
    ActivateDeviceRequest, ActivateDeviceResponse, DeviceChallengeResponse,
    HeartbeatRequest, HeartbeatResponse,
};
use crate::services::license::issue_license_token;

pub async fn create_activation_challenge(
    pool: &PgPool,
    merchant_id: Uuid,
    _device_fingerprint: &str,
) -> Result<DeviceChallengeResponse, ApiError> {
    // Generate challenge
    let challenge = {
        let mut rng = rand::thread_rng();
        (0..32)
            .map(|_| {
                let byte = rng.gen::<u8>();
                format!("{:02x}", byte)
            })
            .collect::<String>()
    };

    // Hash the challenge for storage
    let mut hasher = Sha256::new();
    hasher.update(challenge.as_bytes());
    let challenge_hash = format!("{:x}", hasher.finalize());

    // Set expiration (5 minutes)
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5);

    // Store challenge in database
    let challenge_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO device_activation_challenges (id, merchant_id, challenge_hash, expires_at) VALUES ($1, $2, $3, $4)"
    )
    .bind(challenge_id)
    .bind(merchant_id)
    .bind(&challenge_hash)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(DeviceChallengeResponse {
        challenge_id: challenge_id.to_string(),
        challenge,
        expires_at: expires_at.to_rfc3339(),
    })
}

pub async fn activate_device(
    pool: &PgPool,
    request: ActivateDeviceRequest,
) -> Result<ActivateDeviceResponse, ApiError> {
    // Parse merchant ID
    let merchant_id: Uuid = request.merchant_id.parse()
        .map_err(|_| ApiError::BadRequest("Invalid merchant ID".to_string()))?;

    // Parse challenge ID
    let challenge_id: Uuid = request.challenge_id.parse()
        .map_err(|_| ApiError::BadRequest("Invalid challenge ID".to_string()))?;

    // Validate challenge
    let _challenge = sqlx::query_as::<_, crate::models::Device>(
        "SELECT * FROM device_activation_challenges WHERE id = $1 AND merchant_id = $2 AND consumed_at IS NULL AND expires_at > NOW()"
    )
    .bind(challenge_id)
    .bind(merchant_id)
    .fetch_optional(pool)
    .await?;

    // Note: This is a simplified version. In production, you would:
    // 1. Verify the challenge response signature
    // 2. Check device limit per plan
    // 3. Verify subscription is active

    // Mark challenge as consumed
    sqlx::query(
        "UPDATE device_activation_challenges SET consumed_at = NOW() WHERE id = $1"
    )
    .bind(challenge_id)
    .execute(pool)
    .await?;

    // Hash device identifiers for storage
    let install_id_hash = hash_identifier(&request.install_id);
    let fingerprint_hash = hash_identifier(&request.device_fingerprint);
    let public_key_thumbprint = hash_identifier(&request.device_public_key);

    // Check if device already exists (reactivation)
    let existing_device = sqlx::query_as::<_, crate::models::Device>(
        "SELECT * FROM devices WHERE merchant_id = $1 AND install_id_hash = $2 AND deleted_at IS NULL"
    )
    .bind(merchant_id)
    .bind(&install_id_hash)
    .fetch_optional(pool)
    .await?;

    let device_id = if let Some(device) = existing_device {
        // Update existing device
        sqlx::query(
            "UPDATE devices SET name = $1, device_fingerprint_hash = $2, device_public_key_thumbprint = $3, status = 'active', updated_at = NOW() WHERE id = $4"
        )
        .bind(&request.device_name)
        .bind(&fingerprint_hash)
        .bind(&public_key_thumbprint)
        .bind(device.id)
        .execute(pool)
        .await?;

        device.id
    } else {
        // Create new device
        let device_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO devices (id, merchant_id, name, install_id_hash, device_fingerprint_hash, device_public_key_thumbprint, app_version, os) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind(device_id)
        .bind(merchant_id)
        .bind(&request.device_name)
        .bind(&install_id_hash)
        .bind(&fingerprint_hash)
        .bind(&public_key_thumbprint)
        .bind(&request.app_version)
        .bind(&request.os)
        .execute(pool)
        .await?;

        device_id
    };

    // Get subscription status to determine runtime mode
    let subscription = sqlx::query_as::<_, crate::models::Subscription>(
        "SELECT * FROM subscriptions WHERE merchant_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC LIMIT 1"
    )
    .bind(merchant_id)
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

    // Get active signing key
    let signing_key = sqlx::query_as::<_, crate::models::LicenseSigningKey>(
        "SELECT * FROM license_signing_keys WHERE status = 'active' LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;

    let signing_key = match signing_key {
        Some(key) => key,
        None => return Err(ApiError::InternalError("No active signing key found".to_string())),
    };

    // Issue license token
    let license_token = issue_license_token(
        &device_id,
        &merchant_id,
        &install_id_hash,
        &public_key_thumbprint,
        &signing_key,
        runtime_mode,
        (chrono::Utc::now() + chrono::Duration::days(37)).timestamp(),
    ).await?;

    // Store license in database
    let token_version = 1;
    let token_hash = hash_identifier(&license_token);
    let valid_until = chrono::Utc::now() + chrono::Duration::days(30);
    let grace_until = chrono::Utc::now() + chrono::Duration::days(37);

    sqlx::query(
        "INSERT INTO device_licenses (device_id, token_version, token_hash, signing_key_id, runtime_mode, valid_until, grace_until) VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(device_id)
    .bind(token_version)
    .bind(&token_hash)
    .bind(signing_key.id)
    .bind(runtime_mode)
    .bind(valid_until)
    .bind(grace_until)
    .execute(pool)
    .await?;

    // Log the activation
    sqlx::query(
        "INSERT INTO admin_audit_logs (merchant_id, action, resource_type, resource_id, new_value) VALUES ($1, 'activate_device', 'device', $2, $3)"
    )
    .bind(merchant_id)
    .bind(device_id.to_string())
    .bind(serde_json::json!({
        "device_name": request.device_name,
        "runtime_mode": runtime_mode,
    }))
    .execute(pool)
    .await?;

    tracing::info!("Device {} activated for merchant {}", device_id, merchant_id);

    Ok(ActivateDeviceResponse {
        device_id: device_id.to_string(),
        license_token,
        server_time: chrono::Utc::now().to_rfc3339(),
        runtime_mode: runtime_mode.to_string(),
    })
}

pub async fn process_heartbeat(
    pool: &PgPool,
    request: HeartbeatRequest,
) -> Result<HeartbeatResponse, ApiError> {
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

    // Check nonce replay
    let nonce_hash = hash_identifier(&request.nonce);
    let existing_nonce = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM device_nonces WHERE device_id = $1 AND nonce_hash = $2 AND created_at > NOW() - INTERVAL '5 minutes'"
    )
    .bind(device_id)
    .bind(&nonce_hash)
    .fetch_one(pool)
    .await?;

    if existing_nonce > 0 {
        return Err(ApiError::Forbidden("Nonce replay detected".to_string()));
    }

    // Store nonce
    sqlx::query(
        "INSERT INTO device_nonces (device_id, nonce_hash) VALUES ($1, $2)"
    )
    .bind(device_id)
    .bind(&nonce_hash)
    .execute(pool)
    .await?;

    // Update last heartbeat
    sqlx::query(
        "UPDATE devices SET last_heartbeat_at = NOW(), app_version = $1 WHERE id = $2"
    )
    .bind(&request.app_version)
    .bind(device_id)
    .execute(pool)
    .await?;

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

    // Check if license needs refresh
    let current_license = sqlx::query_as::<_, crate::models::DeviceLicense>(
        "SELECT * FROM device_licenses WHERE device_id = $1 AND revoked_at IS NULL ORDER BY token_version DESC LIMIT 1"
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await?;

    let license_token = match current_license {
        Some(license) => {
            // Check if token is about to expire (within 7 days)
            if license.valid_until < chrono::Utc::now() + chrono::Duration::days(7) {
                // Get signing key
                let signing_key = sqlx::query_as::<_, crate::models::LicenseSigningKey>(
                    "SELECT * FROM license_signing_keys WHERE id = $1"
                )
                .bind(license.signing_key_id)
                .fetch_optional(pool)
                .await?;

                if let Some(key) = signing_key {
                    let new_token = issue_license_token(
                        &device_id,
                        &device.merchant_id,
                        &device.install_id_hash,
                        &device.device_public_key_thumbprint.unwrap_or_default(),
                        &key,
                        runtime_mode,
                        (chrono::Utc::now() + chrono::Duration::days(37)).timestamp(),
                    ).await?;

                    // Store new license
                    let new_version = license.token_version + 1;
                    let token_hash = hash_identifier(&new_token);
                    let valid_until = chrono::Utc::now() + chrono::Duration::days(30);
                    let grace_until = chrono::Utc::now() + chrono::Duration::days(37);

                    sqlx::query(
                        "INSERT INTO device_licenses (device_id, token_version, token_hash, signing_key_id, runtime_mode, valid_until, grace_until) VALUES ($1, $2, $3, $4, $5, $6, $7)"
                    )
                    .bind(device_id)
                    .bind(new_version)
                    .bind(&token_hash)
                    .bind(key.id)
                    .bind(runtime_mode)
                    .bind(valid_until)
                    .bind(grace_until)
                    .execute(pool)
                    .await?;

                    Some(new_token)
                } else {
                    None
                }
            } else {
                None
            }
        }
        None => None,
    };

    Ok(HeartbeatResponse {
        runtime_mode: runtime_mode.to_string(),
        server_time: chrono::Utc::now().to_rfc3339(),
        license_token,
    })
}

fn hash_identifier(identifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(identifier.as_bytes());
    format!("{:x}", hasher.finalize())
}
