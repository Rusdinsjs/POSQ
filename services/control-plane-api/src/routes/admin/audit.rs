use axum::{extract::{Query, State}, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub merchant_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list(
    State(pool): State<PgPool>,
    Query(params): Query<AuditQuery>,
) -> Result<Json<Value>, ApiError> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    let logs = if let Some(merchant_id) = &params.merchant_id {
        let merchant_id: uuid::Uuid = merchant_id.parse()
            .map_err(|_| ApiError::BadRequest("Invalid merchant ID".to_string()))?;

        sqlx::query_as::<_, crate::models::AdminAuditLog>(
            "SELECT * FROM admin_audit_logs WHERE merchant_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(merchant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&pool)
        .await?
    } else {
        sqlx::query_as::<_, crate::models::AdminAuditLog>(
            "SELECT * FROM admin_audit_logs ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&pool)
        .await?
    };

    let response: Vec<Value> = logs.into_iter().map(|l| {
        json!({
            "id": l.id,
            "admin_id": l.admin_id,
            "merchant_id": l.merchant_id,
            "action": l.action,
            "resource_type": l.resource_type,
            "resource_id": l.resource_id,
            "old_value": l.old_value,
            "new_value": l.new_value,
            "ip_address": l.ip_address,
            "created_at": l.created_at,
        })
    }).collect();

    Ok(Json(json!({
        "logs": response,
        "total": response.len(),
        "limit": limit,
        "offset": offset,
    })))
}
