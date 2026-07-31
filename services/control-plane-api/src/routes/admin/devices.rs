use axum::{extract::{Path, State}, Json};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::RevokeDeviceRequest;

pub async fn revoke(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
    Json(payload): Json<RevokeDeviceRequest>,
) -> Result<Json<Value>, ApiError> {
    let device_id: Uuid = id.parse()
        .map_err(|_| ApiError::BadRequest("Invalid device ID".to_string()))?;

    // Check if device exists
    let device = sqlx::query_as::<_, crate::models::Device>(
        "SELECT * FROM devices WHERE id = $1 AND deleted_at IS NULL"
    )
    .bind(device_id)
    .fetch_optional(&pool)
    .await?;

    let device = match device {
        Some(d) => d,
        None => return Err(ApiError::NotFound("Device not found".to_string())),
    };

    if device.status == "revoked" {
        return Err(ApiError::BadRequest("Device is already revoked".to_string()));
    }

    // Revoke device
    sqlx::query(
        "UPDATE devices SET status = 'revoked', updated_at = NOW() WHERE id = $1"
    )
    .bind(device_id)
    .execute(&pool)
    .await?;

    // Revoke all active licenses for this device
    sqlx::query(
        "UPDATE device_licenses SET revoked_at = NOW(), runtime_mode = 'revoked' WHERE device_id = $1 AND revoked_at IS NULL"
    )
    .bind(device_id)
    .execute(&pool)
    .await?;

    // Log the action
    sqlx::query(
        "INSERT INTO admin_audit_logs (merchant_id, action, resource_type, resource_id, new_value) VALUES ($1, 'revoke_device', 'device', $2, $3)"
    )
    .bind(device.merchant_id)
    .bind(device_id.to_string())
    .bind(json!({"reason": payload.reason}))
    .execute(&pool)
    .await?;

    tracing::info!("Device {} revoked by admin", device_id);

    Ok(Json(json!({
        "success": true,
        "message": "Device revoked successfully",
        "server_time": chrono::Utc::now().to_rfc3339()
    })))
}
