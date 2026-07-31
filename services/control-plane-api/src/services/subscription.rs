use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::ManualRenewalRequest;

pub async fn process_manual_renewal(
    pool: &PgPool,
    request: ManualRenewalRequest,
) -> Result<serde_json::Value, ApiError> {
    // Parse IDs
    let merchant_id: Uuid = request.merchant_id.parse()
        .map_err(|_| ApiError::BadRequest("Invalid merchant ID".to_string()))?;

    let subscription_id: Uuid = request.subscription_id.parse()
        .map_err(|_| ApiError::BadRequest("Invalid subscription ID".to_string()))?;

    // Get subscription
    let subscription = sqlx::query_as::<_, crate::models::Subscription>(
        "SELECT * FROM subscriptions WHERE id = $1 AND merchant_id = $2 AND deleted_at IS NULL"
    )
    .bind(subscription_id)
    .bind(merchant_id)
    .fetch_optional(pool)
    .await?;

    let subscription = match subscription {
        Some(s) => s,
        None => return Err(ApiError::NotFound("Subscription not found".to_string())),
    };

    // Calculate new valid_until
    let new_valid_until = if subscription.valid_until > chrono::Utc::now() {
        // Extend from current valid_until
        subscription.valid_until + chrono::Duration::days(request.days_to_add as i64)
    } else {
        // Extend from now if expired
        chrono::Utc::now() + chrono::Duration::days(request.days_to_add as i64)
    };

    // Calculate new grace_until (7 days after valid_until)
    let new_grace_until = new_valid_until + chrono::Duration::days(7);

    // Update subscription
    let new_status = if new_valid_until > chrono::Utc::now() {
        "active"
    } else {
        "restricted_expired"
    };

    sqlx::query(
        "UPDATE subscriptions SET valid_until = $1, grace_until = $2, status = $3, updated_at = NOW() WHERE id = $4"
    )
    .bind(new_valid_until)
    .bind(new_grace_until)
    .bind(new_status)
    .bind(subscription_id)
    .execute(pool)
    .await?;

    // Create subscription event
    let event_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO subscription_events (id, subscription_id, event_type, old_status, new_status, metadata) VALUES ($1, $2, 'manual_renewal', $3, $4, $5)"
    )
    .bind(event_id)
    .bind(subscription_id)
    .bind(&subscription.status)
    .bind(new_status)
    .bind(serde_json::json!({
        "days_added": request.days_to_add,
        "reason": request.reason,
        "old_valid_until": subscription.valid_until,
        "new_valid_until": new_valid_until,
    }))
    .execute(pool)
    .await?;

    // Create admin audit log
    let audit_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO admin_audit_logs (id, merchant_id, action, resource_type, resource_id, old_value, new_value) VALUES ($1, $2, 'manual_renewal', 'subscription', $3, $4, $5)"
    )
    .bind(audit_id)
    .bind(merchant_id)
    .bind(subscription_id.to_string())
    .bind(serde_json::json!({
        "valid_until": subscription.valid_until,
        "status": subscription.status,
    }))
    .bind(serde_json::json!({
        "valid_until": new_valid_until,
        "status": new_status,
        "days_added": request.days_to_add,
        "reason": request.reason,
    }))
    .execute(pool)
    .await?;

    tracing::info!(
        "Subscription {} renewed for merchant {}: {} days added",
        subscription_id,
        merchant_id,
        request.days_to_add
    );

    Ok(serde_json::json!({
        "success": true,
        "subscription_id": subscription_id,
        "old_valid_until": subscription.valid_until,
        "new_valid_until": new_valid_until,
        "new_status": new_status,
        "server_time": chrono::Utc::now().to_rfc3339(),
    }))
}
