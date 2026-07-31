use axum::{extract::State, Json};
use serde_json::Value;
use sqlx::PgPool;

use crate::error::ApiError;
use crate::models::BackupMetadataRequest;
use crate::services::backup::upload_backup_metadata;

pub async fn upload_metadata(
    State(pool): State<PgPool>,
    Json(payload): Json<BackupMetadataRequest>,
) -> Result<Json<Value>, ApiError> {
    // Validate and store backup metadata
    let result = upload_backup_metadata(&pool, payload).await?;

    Ok(Json(result))
}
