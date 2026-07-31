use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::time;

use tracing_subscriber::prelude::*;

mod config;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    // Load configuration
    let config = config::Config::load().expect("Failed to load configuration");

    // Connect to PostgreSQL
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    tracing::info!("Worker started, processing jobs...");

    // Main worker loop
    let mut interval = time::interval(Duration::from_secs(30));

    loop {
        interval.tick().await;

        // Process pending jobs
        match process_pending_jobs(&pool).await {
            Ok(processed) => {
                if processed > 0 {
                    tracing::info!("Processed {} jobs", processed);
                }
            }
            Err(e) => {
                tracing::error!("Error processing jobs: {}", e);
            }
        }

        // Cleanup old idempotency keys
        match cleanup_old_idempotency_keys(&pool).await {
            Ok(cleaned) => {
                if cleaned > 0 {
                    tracing::info!("Cleaned up {} old idempotency keys", cleaned);
                }
            }
            Err(e) => {
                tracing::error!("Error cleaning up idempotency keys: {}", e);
            }
        }

        // Check for license renewals
        match check_license_renewals(&pool).await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Error checking license renewals: {}", e);
            }
        }
    }
}

async fn process_pending_jobs(pool: &sqlx::PgPool) -> Result<i64, sqlx::Error> {
    // Get pending jobs
    let jobs = sqlx::query_as::<_, Job>(
        "SELECT * FROM job_queue WHERE status = 'pending' AND run_after <= NOW() ORDER BY priority DESC LIMIT 10 FOR UPDATE SKIP LOCKED"
    )
    .fetch_all(pool)
    .await?;

    let mut processed = 0;

    for job in jobs {
        // Mark job as processing
        sqlx::query(
            "UPDATE job_queue SET status = 'processing', started_at = NOW() WHERE id = $1"
        )
        .bind(job.id)
        .execute(pool)
        .await?;

        // Process job based on type
        let result = match job.job_type.as_str() {
            "renewal_reminder" => process_renewal_reminder(pool, &job).await,
            "cleanup_expired_challenges" => process_cleanup_challenges(pool).await,
            "cleanup_old_nonces" => process_cleanup_nonces(pool).await,
            _ => {
                tracing::warn!("Unknown job type: {}", job.job_type);
                Ok(())
            }
        };

        // Update job status
        match result {
            Ok(_) => {
                sqlx::query(
                    "UPDATE job_queue SET status = 'completed', completed_at = NOW() WHERE id = $1"
                )
                .bind(job.id)
                .execute(pool)
                .await?;
                processed += 1;
            }
            Err(e) => {
                let new_retry_count = job.retry_count + 1;
                let new_status = if new_retry_count >= job.max_retries {
                    "failed"
                } else {
                    "pending"
                };

                sqlx::query(
                    "UPDATE job_queue SET status = $1, retry_count = $2, last_error = $3, run_after = NOW() + INTERVAL '5 minutes' WHERE id = $4"
                )
                .bind(new_status)
                .bind(new_retry_count)
                .bind(e.to_string())
                .bind(job.id)
                .execute(pool)
                .await?;

                tracing::error!("Job {} failed: {}", job.id, e);
            }
        }
    }

    Ok(processed)
}

async fn process_renewal_reminder(pool: &sqlx::PgPool, _job: &Job) -> Result<(), Box<dyn std::error::Error>> {
    // Find subscriptions expiring in 7 days
    let expiring = sqlx::query_as::<_, Subscription>(
        "SELECT * FROM subscriptions WHERE status = 'active' AND valid_until BETWEEN NOW() AND NOW() + INTERVAL '7 days'"
    )
    .fetch_all(pool)
    .await?;

    for sub in expiring {
        tracing::info!("Subscription {} expiring soon for merchant {}", sub.id, sub.merchant_id);
        // In production, send email/notification here
    }

    Ok(())
}

async fn process_cleanup_challenges(pool: &sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let deleted = sqlx::query(
        "DELETE FROM device_activation_challenges WHERE expires_at < NOW() - INTERVAL '1 day'"
    )
    .execute(pool)
    .await?;

    if deleted.rows_affected() > 0 {
        tracing::info!("Cleaned up {} expired challenges", deleted.rows_affected());
    }

    Ok(())
}

async fn process_cleanup_nonces(pool: &sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let deleted = sqlx::query(
        "DELETE FROM device_nonces WHERE created_at < NOW() - INTERVAL '1 hour'"
    )
    .execute(pool)
    .await?;

    if deleted.rows_affected() > 0 {
        tracing::info!("Cleaned up {} old nonces", deleted.rows_affected());
    }

    Ok(())
}

async fn cleanup_old_idempotency_keys(pool: &sqlx::PgPool) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM idempotency_keys WHERE expires_at < NOW()"
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() as i64)
}

async fn check_license_renewals(pool: &sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // Check for licenses that need refresh
    let licenses = sqlx::query_as::<_, License>(
        "SELECT dl.*, d.merchant_id FROM device_licenses dl JOIN devices d ON dl.device_id = d.id WHERE dl.valid_until < NOW() + INTERVAL '7 days' AND dl.revoked_at IS NULL"
    )
    .fetch_all(pool)
    .await?;

    for license in licenses {
        tracing::info!("License {} for device {} needs refresh", license.id, license.device_id);
        // In production, queue a refresh job here
    }

    Ok(())
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct Job {
    id: uuid::Uuid,
    job_type: String,
    payload: serde_json::Value,
    status: String,
    priority: i32,
    max_retries: i32,
    retry_count: i32,
    last_error: Option<String>,
    run_after: chrono::DateTime<chrono::Utc>,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct Subscription {
    id: uuid::Uuid,
    merchant_id: uuid::Uuid,
    plan_id: uuid::Uuid,
    status: String,
    valid_until: chrono::DateTime<chrono::Utc>,
    grace_until: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct License {
    id: uuid::Uuid,
    device_id: uuid::Uuid,
    valid_until: chrono::DateTime<chrono::Utc>,
    revoked_at: Option<chrono::DateTime<chrono::Utc>>,
}
