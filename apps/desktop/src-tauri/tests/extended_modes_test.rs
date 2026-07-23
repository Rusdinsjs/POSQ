use sqlx::SqlitePool;

#[tokio::test]
async fn test_scale_barcode_parsing() {
    // 20 12345 01500 3 -> SKU 12345, weight 1500g (1.500 kg)
    let barcode = "2012345015003";
    let sku = &barcode[2..7];
    let grams: i32 = barcode[7..12].parse().unwrap();
    let weight_kg = grams as f64 / 1000.0;

    assert_eq!(sku, "12345");
    assert_eq!(grams, 1500);
    assert_eq!(weight_kg, 1.5);
}

#[tokio::test]
async fn test_fefo_lot_selection() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        "CREATE TABLE inventory_lots (
            id TEXT PRIMARY KEY,
            product_id TEXT NOT NULL,
            lot_number TEXT NOT NULL,
            expiry_date TEXT NOT NULL,
            qty_on_hand REAL NOT NULL DEFAULT 0.0,
            supplier_id TEXT,
            created_at TEXT NOT NULL
        );"
    )
    .execute(&pool)
    .await
    .unwrap();

    let now = chrono::Utc::now().to_rfc3339();

    // Insert Lot 1 (expires sooner: 2026-08-01)
    sqlx::query("INSERT INTO inventory_lots (id, product_id, lot_number, expiry_date, qty_on_hand, created_at) VALUES ('lot_1', 'prod_milk', 'LOT-AUG', '2026-08-01', 5.0, ?)")
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    // Insert Lot 2 (expires later: 2026-12-01)
    sqlx::query("INSERT INTO inventory_lots (id, product_id, lot_number, expiry_date, qty_on_hand, created_at) VALUES ('lot_2', 'prod_milk', 'LOT-DEC', '2026-12-01', 10.0, ?)")
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    // Query FEFO lots
    let rows = sqlx::query("SELECT lot_number FROM inventory_lots WHERE product_id = 'prod_milk' AND qty_on_hand > 0 ORDER BY expiry_date ASC")
        .fetch_all(&pool)
        .await
        .unwrap();

    let lot1: String = sqlx::Row::get(&rows[0], "lot_number");
    let lot2: String = sqlx::Row::get(&rows[1], "lot_number");

    assert_eq!(lot1, "LOT-AUG");
    assert_eq!(lot2, "LOT-DEC");
}

#[tokio::test]
async fn test_credit_limit_validation() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        "CREATE TABLE customer_credit_limits (
            customer_id TEXT PRIMARY KEY,
            credit_limit INTEGER NOT NULL DEFAULT 0,
            current_ar INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL
        );"
    )
    .execute(&pool)
    .await
    .unwrap();

    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("INSERT INTO customer_credit_limits (customer_id, credit_limit, current_ar, updated_at) VALUES ('cust_100', 5000000, 4500000, ?)")
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let limit: i32 = sqlx::query_scalar("SELECT credit_limit FROM customer_credit_limits WHERE customer_id = 'cust_100'").fetch_one(&pool).await.unwrap();
    let current_ar: i32 = sqlx::query_scalar("SELECT current_ar FROM customer_credit_limits WHERE customer_id = 'cust_100'").fetch_one(&pool).await.unwrap();

    let additional_sale = 1000000;
    let is_exceeded = (current_ar + additional_sale) > limit;

    assert!(is_exceeded, "AR of 5.5M should exceed 5M credit limit!");
}

#[tokio::test]
async fn test_voucher_pin_masking() {
    let pin1 = "12345678";
    let len1 = pin1.len();
    let masked1 = format!("{}{}****", &pin1[..2], "*".repeat(len1 - 4));
    assert_eq!(masked1, "12****");

    let pin2 = "9999";
    let masked2 = "*".repeat(pin2.len());
    assert_eq!(masked2, "****");
}

#[tokio::test]
async fn test_consignment_commission_calculation() {
    let total_sales = 2000000;
    let commission_percent = 15.0; // 15%
    let commission_amount = (total_sales as f64 * (commission_percent / 100.0)).round() as i32;
    let net_payout = total_sales - commission_amount;

    assert_eq!(commission_amount, 300000);
    assert_eq!(net_payout, 1700000);
}
