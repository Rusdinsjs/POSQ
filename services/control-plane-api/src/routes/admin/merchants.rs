use axum::{extract::{Path, State}, Json};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;

pub async fn list(
    State(pool): State<PgPool>,
) -> Result<Json<Value>, ApiError> {
    let merchants = sqlx::query_as::<_, crate::models::Merchant>(
        "SELECT * FROM merchants WHERE deleted_at IS NULL ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await?;

    let response: Vec<Value> = merchants.into_iter().map(|m| {
        json!({
            "id": m.id,
            "name": m.name,
            "slug": m.slug,
            "email": m.email,
            "phone": m.phone,
            "active": m.active,
            "created_at": m.created_at,
        })
    }).collect();

    Ok(Json(json!({
        "merchants": response,
        "total": response.len(),
    })))
}

pub async fn get(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let merchant_id: Uuid = id.parse()
        .map_err(|_| ApiError::BadRequest("Invalid merchant ID".to_string()))?;

    let merchant = sqlx::query_as::<_, crate::models::Merchant>(
        "SELECT * FROM merchants WHERE id = $1 AND deleted_at IS NULL"
    )
    .bind(merchant_id)
    .fetch_optional(&pool)
    .await?;

    match merchant {
        Some(m) => Ok(Json(json!({
            "id": m.id,
            "name": m.name,
            "slug": m.slug,
            "email": m.email,
            "phone": m.phone,
            "address": m.address,
            "active": m.active,
            "created_at": m.created_at,
        }))),
        None => Err(ApiError::NotFound("Merchant not found".to_string())),
    }
}
