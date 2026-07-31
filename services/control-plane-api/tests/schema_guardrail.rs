use sqlx::{PgPool, Row};

#[tokio::test]
async fn test_schema_guardrail() {
    let pool = PgPool::connect("postgres://pos_app:pos_app_dev@localhost:5432/pos_server")
        .await
        .expect("Failed to connect to pos_server db");

    let tables = sqlx::query("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'")
        .fetch_all(&pool)
        .await
        .expect("Failed to query tables");

    let mut found_forbidden = false;
    let mut forbidden_tables = vec![];

    let forbidden_list = ["orders", "payments", "stock_movements", "inventory_items"];

    for record in tables {
        if let Ok(table_name) = record.try_get::<String, _>("table_name") {
            if forbidden_list.contains(&table_name.as_str()) {
                found_forbidden = true;
                forbidden_tables.push(table_name);
            }
        }
    }

    assert!(
        !found_forbidden,
        "SCHEMA GUARDRAIL FAILED! Server database MUST NOT contain operational POS tables. Found: {:?}",
        forbidden_tables
    );
}
