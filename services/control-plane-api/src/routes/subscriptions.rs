use axum::{extract::State, Json};
use serde_json::Value;
use sqlx::PgPool;

use crate::error::ApiError;
use crate::models::ManualRenewalRequest;
use crate::services::subscription::process_manual_renewal;

pub async fn manual_renewal(
    State(pool): State<PgPool>,
    Json(payload): Json<ManualRenewalRequest>,
) -> Result<Json<Value>, ApiError> {
    // Process manual renewal
    let result = process_manual_renewal(&pool, payload).await?;

    Ok(Json(result))
}
