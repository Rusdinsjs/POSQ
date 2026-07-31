use axum::{extract::State, Json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::{ActivateDeviceRequest, ActivateDeviceResponse, DeviceChallengeRequest, DeviceChallengeResponse, HeartbeatRequest, HeartbeatResponse};
use crate::services::device::{create_activation_challenge, activate_device, process_heartbeat};

pub async fn create_challenge(
    State(pool): State<PgPool>,
    Json(payload): Json<DeviceChallengeRequest>,
) -> Result<Json<DeviceChallengeResponse>, ApiError> {
    // Validate merchant exists
    let merchant_id: Uuid = payload.merchant_id.parse()
        .map_err(|_| ApiError::BadRequest("Invalid merchant ID".to_string()))?;

    let merchant = sqlx::query_as::<_, crate::models::Merchant>(
        "SELECT * FROM merchants WHERE id = $1 AND deleted_at IS NULL"
    )
    .bind(merchant_id)
    .fetch_optional(&pool)
    .await?;

    if merchant.is_none() {
        return Err(ApiError::NotFound("Merchant not found".to_string()));
    }

    // Create challenge
    let challenge = create_activation_challenge(&pool, merchant_id, &payload.device_fingerprint).await?;

    Ok(Json(challenge))
}

pub async fn activate(
    State(pool): State<PgPool>,
    Json(payload): Json<ActivateDeviceRequest>,
) -> Result<Json<ActivateDeviceResponse>, ApiError> {
    // Activate device and get license
    let response = activate_device(&pool, payload).await?;

    Ok(Json(response))
}

pub async fn heartbeat(
    State(pool): State<PgPool>,
    Json(payload): Json<HeartbeatRequest>,
) -> Result<Json<HeartbeatResponse>, ApiError> {
    // Process heartbeat
    let response = process_heartbeat(&pool, payload).await?;

    Ok(Json(response))
}
