use axum::{extract::State, Json};
use sqlx::PgPool;

use crate::error::ApiError;
use crate::models::{RefreshLicenseRequest, RefreshLicenseResponse};
use crate::services::license::refresh_license_token;

pub async fn refresh(
    State(pool): State<PgPool>,
    Json(payload): Json<RefreshLicenseRequest>,
) -> Result<Json<RefreshLicenseResponse>, ApiError> {
    // Refresh license token
    let response = refresh_license_token(&pool, payload).await?;

    Ok(Json(response))
}
