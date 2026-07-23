use sqlx::SqlitePool;
use std::time::Duration;
use reqwest::Client;
use crate::sync_engine::{get_pending_outbox_events, mark_events_pushed, PushBatchRequest, PushBatchResponse};

pub async fn start_sync_worker(pool: SqlitePool) {
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap_or_else(|_| Client::new());

    let mut backoff_secs = 5u64;

    loop {
        // Fetch network settings
        let settings = match crate::settings::get_network_settings_internal(&pool).await {
            Ok(s) => s,
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(30)).await;
                continue;
            }
        };

        if settings.cloud_sync_enabled && !settings.cloud_vps_url.is_empty() {
            match sync_outbox_push(&pool, &client, &settings.cloud_vps_url, &settings.cloud_vps_token).await {
                Ok(pushed_count) => {
                    if pushed_count > 0 {
                        println!("[SyncWorker] Successfully pushed {} outbox events to server.", pushed_count);
                    }
                    backoff_secs = 5; // Reset backoff on success
                }
                Err(err) => {
                    eprintln!("[SyncWorker] Sync push error: {}. Retrying in {}s", err, backoff_secs);
                    tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                    backoff_secs = (backoff_secs * 2).min(300); // Exponential backoff max 5m
                    continue;
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(15)).await;
    }
}

async fn sync_outbox_push(
    pool: &SqlitePool,
    client: &Client,
    base_url: &str,
    token: &str,
) -> Result<usize, String> {
    let pending_events = get_pending_outbox_events(pool, 50).await?;
    if pending_events.is_empty() {
        return Ok(0);
    }

    let push_url = format!("{}/api/v1/sync/push", base_url.trim_end_matches('/'));

    let batch = PushBatchRequest {
        device_id: "desktop_device".into(),
        merchant_id: "default_merchant".into(),
        outlet_id: "default_outlet".into(),
        events: pending_events,
    };

    let resp = client
        .post(&push_url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&batch)
        .send()
        .await
        .map_err(|e| format!("Network request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Server returned HTTP {}", resp.status()));
    }

    let ack_resp: PushBatchResponse = resp.json().await.map_err(|e| format!("Invalid server JSON response: {}", e))?;

    let count = ack_resp.ack_event_ids.len();
    if count > 0 {
        mark_events_pushed(pool, &ack_resp.ack_event_ids).await?;
    }

    Ok(count)
}
