use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, Transaction, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEvent {
    pub id: String,
    pub event_id: String,
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub aggregate_version: i32,
    pub schema_version: i32,
    pub merchant_id: String,
    pub outlet_id: String,
    pub device_id: String,
    pub actor_id: Option<String>,
    pub payload_json: String,
    pub status: String,
    pub retry_count: i32,
    pub last_error: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushBatchRequest {
    pub device_id: String,
    pub merchant_id: String,
    pub outlet_id: String,
    pub events: Vec<OutboxEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushBatchResponse {
    pub ack_event_ids: Vec<String>,
    pub failed_event_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullEventsResponse {
    pub server_cursor: i64,
    pub events: Vec<OutboxEvent>,
}

pub async fn enqueue_outbox_event_tx(
    tx: &mut Transaction<'_, Sqlite>,
    event_type: &str,
    aggregate_type: &str,
    aggregate_id: &str,
    aggregate_version: i32,
    merchant_id: &str,
    outlet_id: &str,
    device_id: &str,
    actor_id: Option<&str>,
    payload_json: &str,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let event_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO sync_outbox (id, event_id, event_type, aggregate_type, aggregate_id, aggregate_version, schema_version, merchant_id, outlet_id, device_id, actor_id, payload_json, status, retry_count, created_at)
         VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?, ?, ?, ?, 'pending', 0, ?)"
    )
    .bind(&id)
    .bind(&event_id)
    .bind(event_type)
    .bind(aggregate_type)
    .bind(aggregate_id)
    .bind(aggregate_version)
    .bind(merchant_id)
    .bind(outlet_id)
    .bind(device_id)
    .bind(actor_id)
    .bind(payload_json)
    .bind(&now)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("Gagal menulis sync_outbox: {}", e))?;

    Ok(event_id)
}

pub async fn get_pending_outbox_events(
    pool: &Pool<Sqlite>,
    limit: i64,
) -> Result<Vec<OutboxEvent>, String> {
    let rows = sqlx::query(
        "SELECT id, event_id, event_type, aggregate_type, aggregate_id, aggregate_version, schema_version, merchant_id, outlet_id, device_id, actor_id, payload_json, status, retry_count, last_error, created_at FROM sync_outbox WHERE status = 'pending' ORDER BY created_at ASC LIMIT ?"
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let events = rows.into_iter().map(|r| OutboxEvent {
        id: r.get("id"),
        event_id: r.get("event_id"),
        event_type: r.get("event_type"),
        aggregate_type: r.get("aggregate_type"),
        aggregate_id: r.get("aggregate_id"),
        aggregate_version: r.get("aggregate_version"),
        schema_version: r.get("schema_version"),
        merchant_id: r.get("merchant_id"),
        outlet_id: r.get("outlet_id"),
        device_id: r.get("device_id"),
        actor_id: r.get("actor_id"),
        payload_json: r.get("payload_json"),
        status: r.get("status"),
        retry_count: r.get("retry_count"),
        last_error: r.get("last_error"),
        created_at: r.get("created_at"),
    }).collect();

    Ok(events)
}

pub async fn mark_events_pushed(
    pool: &Pool<Sqlite>,
    event_ids: &[String],
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    for event_id in event_ids {
        sqlx::query("UPDATE sync_outbox SET status = 'pushed', pushed_at = ? WHERE event_id = ?")
            .bind(&now)
            .bind(event_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn apply_inbound_event(
    pool: &Pool<Sqlite>,
    event: &OutboxEvent,
) -> Result<bool, String> {
    let existing = sqlx::query("SELECT id FROM sync_inbox WHERE event_id = ?")
        .bind(&event.event_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

    if existing.is_some() {
        return Ok(false);
    }

    let inbox_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        "INSERT INTO sync_inbox (id, event_id, event_type, aggregate_type, aggregate_id, payload_json, status, applied_at)
         VALUES (?, ?, ?, ?, ?, ?, 'applied', ?)"
    )
    .bind(&inbox_id)
    .bind(&event.event_id)
    .bind(&event.event_type)
    .bind(&event.aggregate_type)
    .bind(&event.aggregate_id)
    .bind(&event.payload_json)
    .bind(&now)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(true)
}

// Tauri commands
#[tauri::command]
pub async fn get_sync_status_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<serde_json::Value, String> {
    let pending_row = sqlx::query("SELECT COUNT(*) as count FROM sync_outbox WHERE status = 'pending'")
        .fetch_one(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    let pending_count: i64 = pending_row.get("count");

    let pushed_row = sqlx::query("SELECT COUNT(*) as count FROM sync_outbox WHERE status = 'pushed'")
        .fetch_one(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    let pushed_count: i64 = pushed_row.get("count");

    let inbox_row = sqlx::query("SELECT COUNT(*) as count FROM sync_inbox")
        .fetch_one(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    let inbox_count: i64 = inbox_row.get("count");

    Ok(serde_json::json!({
        "pending_outbox_count": pending_count,
        "pushed_outbox_count": pushed_count,
        "inbox_applied_count": inbox_count,
    }))
}
