use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::BackupMetadataRequest;

pub async fn upload_backup_metadata(
    pool: &PgPool,
    request: BackupMetadataRequest,
) -> Result<serde_json::Value, ApiError> {
    // Parse device ID
    let device_id: Uuid = request.device_id.parse()
        .map_err(|_| ApiError::BadRequest("Invalid device ID".to_string()))?;

    // Get device to verify merchant ownership
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

    // Validate backup is encrypted
    if !request.encrypted {
        return Err(ApiError::BadRequest("Backup must be encrypted".to_string()));
    }

    // Validate size (max 100MB for metadata)
    if request.size_bytes > 100 * 1024 * 1024 {
        return Err(ApiError::BadRequest("Backup size exceeds maximum limit".to_string()));
    }

    // Check for idempotency (if backup_id already exists)
    let existing = sqlx::query_as::<_, crate::models::BackupMetadata>(
        "SELECT * FROM backup_metadata WHERE backup_id = $1"
    )
    .bind(&request.backup_id)
    .fetch_optional(pool)
    .await?;

    if let Some(existing) = existing {
        // Return existing metadata (idempotent response)
        return Ok(serde_json::json!({
            "success": true,
            "backup_id": existing.backup_id,
            "status": existing.status,
            "created_at": existing.created_at,
            "server_time": chrono::Utc::now().to_rfc3339(),
        }));
    }

    // Store backup metadata
    let backup_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO backup_metadata (id, merchant_id, device_id, backup_id, destination_type, logical_storage_ref, size_bytes, checksum, encryption_algorithm, encrypted, app_version, db_schema_version, status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'uploaded')"
    )
    .bind(backup_id)
    .bind(device.merchant_id)
    .bind(device_id)
    .bind(&request.backup_id)
    .bind(&request.destination_type)
    .bind(&request.logical_storage_ref)
    .bind(request.size_bytes)
    .bind(&request.checksum)
    .bind(&request.encryption_algorithm)
    .bind(request.encrypted)
    .bind(&request.app_version)
    .bind(&request.db_schema_version)
    .execute(pool)
    .await?;

    tracing::info!(
        "Backup metadata uploaded: {} ({} bytes) for device {}",
        request.backup_id,
        request.size_bytes,
        device_id
    );

    Ok(serde_json::json!({
        "success": true,
        "backup_id": request.backup_id,
        "status": "uploaded",
        "server_time": chrono::Utc::now().to_rfc3339(),
    }))
}
