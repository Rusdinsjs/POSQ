use sqlx::SqlitePool;

#[tokio::test]
async fn test_capability_resolution_and_preset_switch() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Create sync and capability tables
    sqlx::query(
        "CREATE TABLE outlet_profiles (
            outlet_id TEXT PRIMARY KEY,
            primary_preset_code TEXT NOT NULL,
            preset_version INTEGER NOT NULL DEFAULT 1,
            config_version INTEGER NOT NULL DEFAULT 1,
            activated_at TEXT NOT NULL,
            activated_by TEXT
        );"
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE outlet_capabilities (
            outlet_id TEXT NOT NULL,
            capability_key TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            source TEXT NOT NULL DEFAULT 'preset',
            config_json TEXT NOT NULL DEFAULT '{}',
            updated_at TEXT NOT NULL,
            updated_by TEXT,
            PRIMARY KEY (outlet_id, capability_key)
        );"
    )
    .execute(&pool)
    .await
    .unwrap();

    // 1. Initial resolution (default general_flexible)
    let initial_effective = sqlx::query("SELECT primary_preset_code FROM outlet_profiles WHERE outlet_id = 'outlet_1'")
        .fetch_optional(&pool)
        .await
        .unwrap();
    assert!(initial_effective.is_none());

    // 2. Set preset to fnb_table_service
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO outlet_profiles (outlet_id, primary_preset_code, preset_version, config_version, activated_at, activated_by)
         VALUES ('outlet_1', 'fnb_table_service', 1, 1, ?, 'admin')"
    )
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    let profile = sqlx::query("SELECT primary_preset_code FROM outlet_profiles WHERE outlet_id = 'outlet_1'")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    let code: String = sqlx::Row::get(&profile, "primary_preset_code");
    assert_eq!(code, "fnb_table_service");
}

#[tokio::test]
async fn test_sync_outbox_schema_and_idempotency() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        "CREATE TABLE sync_outbox (
            id TEXT PRIMARY KEY,
            event_id TEXT UNIQUE NOT NULL,
            event_type TEXT NOT NULL,
            aggregate_type TEXT NOT NULL,
            aggregate_id TEXT NOT NULL,
            aggregate_version INTEGER NOT NULL DEFAULT 1,
            schema_version INTEGER NOT NULL DEFAULT 1,
            merchant_id TEXT NOT NULL DEFAULT 'default_merchant',
            outlet_id TEXT NOT NULL DEFAULT 'default_outlet',
            device_id TEXT NOT NULL DEFAULT 'default_device',
            actor_id TEXT,
            payload_json TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            retry_count INTEGER NOT NULL DEFAULT 0,
            last_error TEXT,
            created_at TEXT NOT NULL,
            pushed_at TEXT
        );"
    )
    .execute(&pool)
    .await
    .unwrap();

    let now = chrono::Utc::now().to_rfc3339();
    let event_id = uuid::Uuid::new_v4().to_string();

    // Insert outbox event
    sqlx::query(
        "INSERT INTO sync_outbox (id, event_id, event_type, aggregate_type, aggregate_id, payload_json, status, created_at)
         VALUES ('1', ?, 'ORDER_CREATED', 'order', 'ord_123', '{\"total\": 50000}', 'pending', ?)"
    )
    .bind(&event_id)
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    // Verify event is present
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sync_outbox WHERE status = 'pending'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);

    // Verify duplicate event_id insert fails due to UNIQUE constraint (Idempotency)
    let dup_res = sqlx::query(
        "INSERT INTO sync_outbox (id, event_id, event_type, aggregate_type, aggregate_id, payload_json, status, created_at)
         VALUES ('2', ?, 'ORDER_CREATED', 'order', 'ord_123', '{\"total\": 50000}', 'pending', ?)"
    )
    .bind(&event_id)
    .bind(&now)
    .execute(&pool)
    .await;

    assert!(dup_res.is_err(), "Duplicate event_id insertion must fail!");
}
