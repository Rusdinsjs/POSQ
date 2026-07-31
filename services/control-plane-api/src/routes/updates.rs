use axum::{extract::{State, Query}, Json};
use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;

use crate::error::ApiError;
use crate::models::{UpdateCheckRequest, UpdateCheckResponse, PublishUpdateRequest};
use crate::services::update::{check_for_update, publish_update_metadata};

#[derive(Debug, Deserialize)]
pub struct UpdateCheckQuery {
    pub os: String,
    pub channel: String,
    pub version: String,
}

pub async fn check(
    State(pool): State<PgPool>,
    Query(params): Query<UpdateCheckQuery>,
) -> Result<Json<UpdateCheckResponse>, ApiError> {
    let request = UpdateCheckRequest {
        os: params.os,
        channel: params.channel,
        current_version: params.version,
    };

    let response = check_for_update(&pool, request).await?;

    Ok(Json(response))
}

pub async fn publish(
    State(pool): State<PgPool>,
    Json(payload): Json<PublishUpdateRequest>,
) -> Result<Json<Value>, ApiError> {
    // In production, this would require admin authentication and MFA
    let result = publish_update_metadata(&pool, payload).await?;

    Ok(Json(result))
}
