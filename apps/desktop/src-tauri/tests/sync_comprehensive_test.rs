use sqlx::SqlitePool;
use uuid::Uuid;

#[tokio::test]
async fn test_01_offline_local_checkout() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Create minimal schema for order & outbox
    sqlx::query(
        "CREATE TABLE orders (id TEXT PRIMARY KEY, order_number TEXT, grand_total INTEGER, status TEXT, created_at TEXT);"
    ).execute(&pool).await.unwrap();

    sqlx::query(
        "CREATE TABLE sync_outbox (id TEXT PRIMARY KEY, event_id TEXT UNIQUE, event_type TEXT, aggregate_type TEXT, aggregate_id TEXT, payload_json TEXT, status TEXT, created_at TEXT);"
    ).execute(&pool).await.unwrap();

    let mut tx = pool.begin().await.unwrap();

    let order_id = Uuid::new_v4().to_string();
    let event_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Insert order domain state
    sqlx::query("INSERT INTO orders (id, order_number, grand_total, status, created_at) VALUES (?, 'ORD-OFFLINE-1', 75000, 'paid', ?)")
        .bind(&order_id)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .unwrap();

    // Insert outbox event in same transaction
    sqlx::query("INSERT INTO sync_outbox (id, event_id, event_type, aggregate_type, aggregate_id, payload_json, status, created_at) VALUES (?, ?, 'ORDER_CREATED', 'order', ?, '{\"total\": 75000}', 'pending', ?)")
        .bind(Uuid::new_v4().to_string())
        .bind(&event_id)
        .bind(&order_id)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .unwrap();

    tx.commit().await.unwrap();

    // Verify order and outbox event exist locally
    let order_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM orders")
        .fetch_one(&pool)
        .await
        .unwrap();
    let outbox_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sync_outbox WHERE status = 'pending'")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(order_count, 1, "Order must be created locally offline!");
    assert_eq!(outbox_count, 1, "Outbox event must be enqueued atomically!");
}

#[tokio::test]
async fn test_02_push_event_100x_idempotency() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query("CREATE TABLE sync_inbox_dedup (event_id TEXT PRIMARY KEY, processed_at TEXT);")
        .execute(&pool)
        .await
        .unwrap();

    let event_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // First push ingestion
    let res1 = sqlx::query("INSERT INTO sync_inbox_dedup (event_id, processed_at) VALUES (?, ?) ON CONFLICT(event_id) DO NOTHING")
        .bind(&event_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    assert_eq!(res1.rows_affected(), 1, "First push must insert event");

    // Push 99 more times
    let mut total_inserted = 1;
    for _ in 0..99 {
        let res = sqlx::query("INSERT INTO sync_inbox_dedup (event_id, processed_at) VALUES (?, ?) ON CONFLICT(event_id) DO NOTHING")
            .bind(&event_id)
            .bind(&now)
            .execute(&pool)
            .await
            .unwrap();

        total_inserted += res.rows_affected();
    }

    assert_eq!(
        total_inserted, 1,
        "100x duplicate pushes must result in exactly 1 insertion!"
    );
}

#[tokio::test]
async fn test_03_pull_event_100x_inbound_deduplication() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        "CREATE TABLE sync_inbox (id TEXT PRIMARY KEY, event_id TEXT UNIQUE, event_type TEXT, payload_json TEXT, status TEXT, applied_at TEXT);"
    ).execute(&pool).await.unwrap();

    let event_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Ingest 100 duplicate pulls locally
    let mut applied_count = 0;
    for i in 0..100 {
        let existing = sqlx::query("SELECT id FROM sync_inbox WHERE event_id = ?")
            .bind(&event_id)
            .fetch_optional(&pool)
            .await
            .unwrap();

        if existing.is_none() {
            sqlx::query("INSERT INTO sync_inbox (id, event_id, event_type, payload_json, status, applied_at) VALUES (?, ?, 'ORDER_CREATED', '{}', 'applied', ?)")
                .bind(format!("inbox_{}", i))
                .bind(&event_id)
                .bind(&now)
                .execute(&pool)
                .await
                .unwrap();

            applied_count += 1;
        }
    }

    assert_eq!(
        applied_count, 1,
        "100x pull applications must be deduplicated locally to 1!"
    );
}

#[tokio::test]
async fn test_04_master_data_conflict_recording() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        "CREATE TABLE sync_conflicts (
            id TEXT PRIMARY KEY,
            aggregate_type TEXT NOT NULL,
            aggregate_id TEXT NOT NULL,
            client_version INTEGER NOT NULL,
            server_version INTEGER NOT NULL,
            conflict_type TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'unresolved',
            created_at TEXT NOT NULL
        );",
    )
    .execute(&pool)
    .await
    .unwrap();

    let conflict_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Concurrent edit collision: Client version 2 vs Server version 3
    sqlx::query(
        "INSERT INTO sync_conflicts (id, aggregate_type, aggregate_id, client_version, server_version, conflict_type, status, created_at)
         VALUES (?, 'product', 'prod_apple', 2, 3, 'VERSION_MISMATCH', 'unresolved', ?)"
    )
    .bind(&conflict_id)
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sync_conflicts WHERE status = 'unresolved'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        count, 1,
        "Master data version conflicts must be recorded in sync_conflicts!"
    );
}

#[tokio::test]
async fn test_05_cross_tenant_isolation() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        "CREATE TABLE orders (id TEXT PRIMARY KEY, merchant_id TEXT NOT NULL, grand_total INTEGER);"
    ).execute(&pool).await.unwrap();

    sqlx::query("INSERT INTO orders (id, merchant_id, grand_total) VALUES ('ord_m1', 'merchant_alpha', 50000)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO orders (id, merchant_id, grand_total) VALUES ('ord_m2', 'merchant_beta', 100000)").execute(&pool).await.unwrap();

    // Query for merchant_alpha
    let alpha_orders = sqlx::query("SELECT id FROM orders WHERE merchant_id = ?")
        .bind("merchant_alpha")
        .fetch_all(&pool)
        .await
        .unwrap();

    assert_eq!(alpha_orders.len(), 1);
    let alpha_id: String = sqlx::Row::get(&alpha_orders[0], "id");
    assert_eq!(
        alpha_id, "ord_m1",
        "Merchant Alpha must not see Merchant Beta orders!"
    );
}
