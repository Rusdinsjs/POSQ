use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonationRecord {
    pub id: String,
    pub order_id: String,
    pub donor_name: String,
    pub donor_phone: Option<String>,
    pub campaign_name: String,
    pub fund_type: String,
    pub amount: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalWarehouseIssue {
    pub id: String,
    pub cost_center: String,
    pub requester_name: String,
    pub product_id: String,
    pub qty: f64,
    pub unit_cost: i32,
    pub issued_at: String,
}

pub async fn record_donation(
    pool: &Pool<Sqlite>,
    order_id: &str,
    donor_name: &str,
    donor_phone: Option<&str>,
    campaign_name: &str,
    fund_type: &str,
    amount: i32,
) -> Result<DonationRecord, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO donation_records (id, order_id, donor_name, donor_phone, campaign_name, fund_type, amount, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(order_id)
    .bind(donor_name)
    .bind(donor_phone)
    .bind(campaign_name)
    .bind(fund_type)
    .bind(amount)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("Gagal mencatat penerimaan donasi: {}", e))?;

    Ok(DonationRecord {
        id,
        order_id: order_id.to_string(),
        donor_name: donor_name.to_string(),
        donor_phone: donor_phone.map(|s| s.to_string()),
        campaign_name: campaign_name.to_string(),
        fund_type: fund_type.to_string(),
        amount,
        created_at: now,
    })
}

pub fn check_public_service_discount_guard(discount_amount: i32) -> Result<(), String> {
    if discount_amount > 0 {
        return Err("NO_DISCOUNT_ALLOWED: Mode Layanan Publik / Retribusi tidak mengizinkan potongan diskon".into());
    }
    Ok(())
}

pub async fn issue_internal_inventory(
    pool: &Pool<Sqlite>,
    cost_center: &str,
    requester_name: &str,
    product_id: &str,
    qty: f64,
    unit_cost: i32,
) -> Result<InternalWarehouseIssue, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // Deduct inventory without revenue
    sqlx::query("UPDATE inventory_items SET qty_on_hand = qty_on_hand - ? WHERE product_id = ?")
        .bind(qty)
        .bind(product_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query(
        "INSERT INTO internal_warehouse_issues (id, cost_center, requester_name, product_id, qty, unit_cost, issued_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(cost_center)
    .bind(requester_name)
    .bind(product_id)
    .bind(qty)
    .bind(unit_cost)
    .bind(&now)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(InternalWarehouseIssue {
        id,
        cost_center: cost_center.to_string(),
        requester_name: requester_name.to_string(),
        product_id: product_id.to_string(),
        qty,
        unit_cost,
        issued_at: now,
    })
}

// Tauri commands
#[tauri::command]
pub async fn record_donation_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
    order_id: String,
    donor_name: String,
    donor_phone: Option<String>,
    campaign_name: String,
    fund_type: String,
    amount: i32,
) -> Result<DonationRecord, String> {
    record_donation(&pool, &order_id, &donor_name, donor_phone.as_deref(), &campaign_name, &fund_type, amount).await
}

#[tauri::command]
pub async fn issue_internal_inventory_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
    cost_center: String,
    requester_name: String,
    product_id: String,
    qty: f64,
    unit_cost: i32,
) -> Result<InternalWarehouseIssue, String> {
    issue_internal_inventory(&pool, &cost_center, &requester_name, &product_id, qty, unit_cost).await
}
