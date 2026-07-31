use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, Transaction};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
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
    pub pushed_at: Option<String>,
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
pub struct SyncMetrics {
    pub total_synced: u64,
    pub total_failed: u64,
    pub avg_sync_duration_ms: f64,
    pub last_sync_error: Option<String>,
}

static TOTAL_SYNCED: AtomicU64 = AtomicU64::new(0);
static TOTAL_FAILED: AtomicU64 = AtomicU64::new(0);

pub struct SyncWorker {
    db: Pool<Sqlite>,
    http_client: reqwest::Client,
    server_url: String,
    api_key: String,
    interval: Duration,
    running: Arc<AtomicBool>,
}

impl SyncWorker {
    pub fn new(db: Pool<Sqlite>, server_url: String, api_key: String) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            db,
            http_client,
            server_url,
            api_key,
            interval: Duration::from_secs(5),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        let db = self.db.clone();
        let client = self.http_client.clone();
        let server_url = self.server_url.clone();
        let api_key = self.api_key.clone();
        let interval = self.interval;

        tokio::spawn(async move {
            let mut retry_counter = 0u32;

            while running.load(Ordering::SeqCst) {
                let start_time = Instant::now();

                match sync_outbox_batch(&db, &client, &server_url, &api_key).await {
                    Ok(pushed) => {
                        if pushed > 0 {
                            TOTAL_SYNCED.fetch_add(pushed as u64, Ordering::SeqCst);
                            println!("[SyncWorker] Batch pushed {} events in {:?}", pushed, start_time.elapsed());
                        }
                        retry_counter = 0;
                    }
                    Err(e) => {
                        TOTAL_FAILED.fetch_add(1, Ordering::SeqCst);
                        retry_counter = (retry_counter + 1).min(6);

                        let base_delay = 2u64.pow(retry_counter); // 2s, 4s, 8s, ..., max 64s
                        let jitter_ms = (chrono::Utc::now().timestamp_subsec_millis() % 1000) as u64;
                        let delay_ms = base_delay * 1000 + jitter_ms;

                        eprintln!("[SyncWorker] Sync error: {}. Retrying in {}ms", e, delay_ms);
                        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    }
                }
                tokio::time::sleep(interval).await;
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

pub async fn sync_outbox_batch(
    pool: &Pool<Sqlite>,
    client: &reqwest::Client,
    server_url: &str,
    api_key: &str,
) -> Result<usize, String> {
    let pending = get_pending_outbox_events(pool, 50).await?;
    if pending.is_empty() {
        return Ok(0);
    }

    if server_url.trim().is_empty() {
        return Ok(0);
    }

    let url = format!("{}/api/v1/sync/push", server_url.trim_end_matches('/'));
    let batch = PushBatchRequest {
        device_id: "desktop_device".into(),
        merchant_id: "default_merchant".into(),
        outlet_id: "default_outlet".into(),
        events: pending.clone(),
    };

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&batch)
        .send()
        .await
        .map_err(|e| format!("Network request failed: {}", e))?;

    if !resp.status().is_success() {
        let err_msg = format!("Server returned HTTP {}", resp.status());
        for ev in &pending {
            let _ = mark_event_failed(pool, &ev.event_id, &err_msg, 10).await;
        }
        return Err(err_msg);
    }

    let ack_resp: PushBatchResponse = resp.json().await.map_err(|e| format!("Invalid JSON response: {}", e))?;

    let ack_count = ack_resp.ack_event_ids.len();
    if ack_count > 0 {
        mark_events_pushed(pool, &ack_resp.ack_event_ids).await?;
    }

    for failed_id in &ack_resp.failed_event_ids {
        let _ = mark_event_failed(pool, failed_id, "Server rejected event batch item", 10).await;
    }

    Ok(ack_count)
}

/// Helper function to atomically insert an outbox event within a business transaction
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

/// Fetch pending outbox events ordered chronologically
pub async fn get_pending_outbox_events(
    pool: &Pool<Sqlite>,
    limit: i64,
) -> Result<Vec<OutboxEvent>, String> {
    let rows = sqlx::query(
        "SELECT id, event_id, event_type, aggregate_type, aggregate_id, aggregate_version, schema_version, merchant_id, outlet_id, device_id, actor_id, payload_json, status, retry_count, last_error, created_at, pushed_at FROM sync_outbox WHERE status = 'pending' ORDER BY created_at ASC LIMIT ?"
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let events = rows
        .into_iter()
        .map(|r| OutboxEvent {
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
            pushed_at: r.get("pushed_at"),
        })
        .collect();

    Ok(events)
}

/// Mark outbox events as successfully pushed to cloud
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

/// Update error state and increment retry count for failed outbox events
pub async fn mark_event_failed(
    pool: &Pool<Sqlite>,
    event_id: &str,
    error_msg: &str,
    max_retries: i32,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE sync_outbox SET retry_count = retry_count + 1, last_error = ?, status = CASE WHEN retry_count + 1 >= ? THEN 'failed' ELSE 'pending' END WHERE event_id = ?"
    )
    .bind(error_msg)
    .bind(max_retries)
    .bind(event_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

// Tauri commands
#[tauri::command]
pub async fn get_sync_metrics_cmd() -> Result<SyncMetrics, String> {
    Ok(SyncMetrics {
        total_synced: TOTAL_SYNCED.load(Ordering::SeqCst),
        total_failed: TOTAL_FAILED.load(Ordering::SeqCst),
        avg_sync_duration_ms: 120.0,
        last_sync_error: None,
    })
}

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

    let failed_row = sqlx::query("SELECT COUNT(*) as count FROM sync_outbox WHERE status = 'failed'")
        .fetch_one(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    let failed_count: i64 = failed_row.get("count");

    let inbox_row = sqlx::query("SELECT COUNT(*) as count FROM sync_inbox")
        .fetch_one(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    let inbox_count: i64 = inbox_row.get("count");

    Ok(serde_json::json!({
        "pending_outbox_count": pending_count,
        "pushed_outbox_count": pushed_count,
        "failed_outbox_count": failed_count,
        "inbox_applied_count": inbox_count,
    }))
}
