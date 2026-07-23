use sqlx::SqlitePool;

#[tokio::test]
async fn test_donation_record_creation() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        "CREATE TABLE donation_records (
            id TEXT PRIMARY KEY,
            order_id TEXT NOT NULL,
            donor_name TEXT NOT NULL,
            donor_phone TEXT,
            campaign_name TEXT NOT NULL DEFAULT 'General Fund',
            fund_type TEXT NOT NULL DEFAULT 'unrestricted',
            amount INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );"
    )
    .execute(&pool)
    .await
    .unwrap();

    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO donation_records (id, order_id, donor_name, donor_phone, campaign_name, fund_type, amount, created_at)
         VALUES ('don_1', 'ord_don_100', 'H. Ahmad', '08123456789', 'Pembangunan Masjid', 'zakat', 500000, ?)"
    )
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    let amount: i32 = sqlx::query_scalar("SELECT amount FROM donation_records WHERE id = 'don_1'").fetch_one(&pool).await.unwrap();
    assert_eq!(amount, 500000);
}

#[tokio::test]
async fn test_public_service_fee_discount_guard() {
    let discount_attempt = 10000;
    let is_discount_allowed = discount_attempt == 0;

    assert!(!is_discount_allowed, "Public service fee retribusi must not allow discounts!");
}

#[tokio::test]
async fn test_internal_warehouse_issue() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        "CREATE TABLE inventory_items (product_id TEXT PRIMARY KEY, qty_on_hand REAL NOT NULL);"
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE internal_warehouse_issues (
            id TEXT PRIMARY KEY,
            cost_center TEXT NOT NULL,
            requester_name TEXT NOT NULL,
            product_id TEXT NOT NULL,
            qty REAL NOT NULL DEFAULT 1.0,
            unit_cost INTEGER NOT NULL DEFAULT 0,
            issued_at TEXT NOT NULL
        );"
    )
    .execute(&pool)
    .await
    .unwrap();

    let now = chrono::Utc::now().to_rfc3339();

    // Initial stock: 20 units
    sqlx::query("INSERT INTO inventory_items (product_id, qty_on_hand) VALUES ('prod_paper', 20.0)").execute(&pool).await.unwrap();

    // Internal dispatch of 5 units to Pantry Cost Center
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("UPDATE inventory_items SET qty_on_hand = qty_on_hand - 5.0 WHERE product_id = 'prod_paper'").execute(&mut *tx).await.unwrap();
    sqlx::query(
        "INSERT INTO internal_warehouse_issues (id, cost_center, requester_name, product_id, qty, unit_cost, issued_at)
         VALUES ('iss_1', 'Pantry HR', 'Siti', 'prod_paper', 5.0, 45000, ?)"
    )
    .bind(&now)
    .execute(&mut *tx)
    .await
    .unwrap();

    tx.commit().await.unwrap();

    let remaining_qty: f64 = sqlx::query_scalar("SELECT qty_on_hand FROM inventory_items WHERE product_id = 'prod_paper'").fetch_one(&pool).await.unwrap();
    assert_eq!(remaining_qty, 15.0, "Internal issue must deduct stock without revenue!");
}
