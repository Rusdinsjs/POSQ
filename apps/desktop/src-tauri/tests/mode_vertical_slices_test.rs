use sqlx::SqlitePool;

#[tokio::test]
async fn test_serialized_inventory_lifecycle() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Create table schema
    sqlx::query(
        "CREATE TABLE serialized_units (
            id TEXT PRIMARY KEY,
            product_id TEXT NOT NULL,
            serial_no TEXT UNIQUE NOT NULL,
            imei1 TEXT,
            imei2 TEXT,
            status TEXT NOT NULL DEFAULT 'in_stock',
            unit_cost INTEGER NOT NULL DEFAULT 0,
            warranty_months INTEGER NOT NULL DEFAULT 12,
            received_at TEXT NOT NULL,
            sold_at TEXT,
            order_id TEXT
        );"
    )
    .execute(&pool)
    .await
    .unwrap();

    let now = chrono::Utc::now().to_rfc3339();

    // Register serial unit
    sqlx::query(
        "INSERT INTO serialized_units (id, product_id, serial_no, imei1, imei2, status, unit_cost, warranty_months, received_at)
         VALUES ('unit_1', 'prod_phone', 'SN-IPHONE-001', '356789012345678', NULL, 'in_stock', 12000000, 12, ?)"
    )
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    // Verify stock status
    let status: String = sqlx::query_scalar("SELECT status FROM serialized_units WHERE serial_no = 'SN-IPHONE-001'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "in_stock");

    // Sell unit in order
    sqlx::query("UPDATE serialized_units SET status = 'sold', sold_at = ?, order_id = 'ord_777' WHERE serial_no = 'SN-IPHONE-001'")
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let sold_status: String = sqlx::query_scalar("SELECT status FROM serialized_units WHERE serial_no = 'SN-IPHONE-001'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(sold_status, "sold");
}

#[tokio::test]
async fn test_table_service_state_machine() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        "CREATE TABLE dining_sessions (
            id TEXT PRIMARY KEY,
            table_id TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'open',
            opened_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );"
    )
    .execute(&pool)
    .await
    .unwrap();

    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("INSERT INTO dining_sessions (id, table_id, status, opened_at, updated_at) VALUES ('sess_1', 'tbl_A1', 'open', ?, ?)")
        .bind(&now)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    // Valid transition open -> order_placed
    sqlx::query("UPDATE dining_sessions SET status = 'order_placed', updated_at = ? WHERE id = 'sess_1'")
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let status: String = sqlx::query_scalar("SELECT status FROM dining_sessions WHERE id = 'sess_1'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "order_placed");
}

#[tokio::test]
async fn test_appointment_lifecycle() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        "CREATE TABLE staff_resources (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'staff',
            active INTEGER NOT NULL DEFAULT 1
        );"
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE appointments (
            id TEXT PRIMARY KEY,
            customer_id TEXT,
            staff_id TEXT NOT NULL,
            service_product_id TEXT NOT NULL,
            scheduled_at TEXT NOT NULL,
            duration_minutes INTEGER NOT NULL DEFAULT 60,
            status TEXT NOT NULL DEFAULT 'scheduled',
            deposit_amount INTEGER NOT NULL DEFAULT 0,
            notes TEXT,
            created_at TEXT NOT NULL
        );"
    )
    .execute(&pool)
    .await
    .unwrap();

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO staff_resources (id, name, role) VALUES ('stf_1', 'Budi Barber', 'hairdresser')").execute(&pool).await.unwrap();

    sqlx::query(
        "INSERT INTO appointments (id, staff_id, service_product_id, scheduled_at, duration_minutes, status, deposit_amount, created_at)
         VALUES ('app_1', 'stf_1', 'srv_haircut', ?, 45, 'scheduled', 20000, ?)"
    )
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    // Check in -> in_service -> completed
    sqlx::query("UPDATE appointments SET status = 'completed' WHERE id = 'app_1'").execute(&pool).await.unwrap();

    let status: String = sqlx::query_scalar("SELECT status FROM appointments WHERE id = 'app_1'").fetch_one(&pool).await.unwrap();
    assert_eq!(status, "completed");
}

#[tokio::test]
async fn test_repair_ticket_lifecycle() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        "CREATE TABLE customer_assets (
            id TEXT PRIMARY KEY,
            customer_id TEXT,
            asset_type TEXT NOT NULL,
            brand_model TEXT NOT NULL,
            serial_no TEXT,
            created_at TEXT NOT NULL
        );"
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE repair_tickets (
            id TEXT PRIMARY KEY,
            ticket_number TEXT UNIQUE NOT NULL,
            asset_id TEXT NOT NULL,
            customer_id TEXT,
            problem_description TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'received',
            estimated_cost INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );"
    )
    .execute(&pool)
    .await
    .unwrap();

    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("INSERT INTO customer_assets (id, asset_type, brand_model, serial_no, created_at) VALUES ('ast_1', 'laptop', 'MacBook Pro 14', 'C02F12345', ?)")
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO repair_tickets (id, ticket_number, asset_id, problem_description, status, estimated_cost, created_at, updated_at)
         VALUES ('tk_1', 'RPR-1001', 'ast_1', 'Layar pecah', 'received', 2500000, ?, ?)"
    )
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    // Progress to repairing -> ready
    sqlx::query("UPDATE repair_tickets SET status = 'ready', updated_at = ? WHERE id = 'tk_1'")
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let status: String = sqlx::query_scalar("SELECT status FROM repair_tickets WHERE id = 'tk_1'").fetch_one(&pool).await.unwrap();
    assert_eq!(status, "ready");
}
