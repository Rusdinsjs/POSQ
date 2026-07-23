use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsignmentSettlement {
    pub id: String,
    pub consignor_id: String,
    pub period_start: String,
    pub period_end: String,
    pub total_sales: i32,
    pub commission_amount: i32,
    pub net_payout: i32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskedDigitalVoucher {
    pub voucher_code: String,
    pub masked_pin: String,
    pub status: String,
}

pub fn mask_voucher_secret(raw_pin: &str) -> String {
    let len = raw_pin.len();
    if len <= 4 {
        return "*".repeat(len);
    }
    format!("{}{}****", &raw_pin[..2], "*".repeat(len - 4))
}

pub async fn calculate_consignor_settlement(
    pool: &Pool<Sqlite>,
    consignor_id: &str,
    period_start: &str,
    period_end: &str,
    commission_percent: f64,
) -> Result<ConsignmentSettlement, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Query sales for this consignor in given period
    let row = sqlx::query("SELECT COALESCE(SUM(line_total), 0) as total FROM order_items WHERE notes LIKE ?")
        .bind(format!("%consignor:{}%", consignor_id))
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    let total_sales: i32 = row.get("total");
    let commission_amount = (total_sales as f64 * (commission_percent / 100.0)).round() as i32;
    let net_payout = total_sales - commission_amount;

    sqlx::query(
        "INSERT INTO consignment_settlements (id, consignor_id, period_start, period_end, total_sales, commission_amount, net_payout, status, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, 'draft', ?)"
    )
    .bind(&id)
    .bind(consignor_id)
    .bind(period_start)
    .bind(period_end)
    .bind(total_sales)
    .bind(commission_amount)
    .bind(net_payout)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(ConsignmentSettlement {
        id,
        consignor_id: consignor_id.to_string(),
        period_start: period_start.to_string(),
        period_end: period_end.to_string(),
        total_sales,
        commission_amount,
        net_payout,
        status: "draft".to_string(),
    })
}

// Tauri commands
#[tauri::command]
pub fn mask_voucher_pin_cmd(raw_pin: String) -> String {
    mask_voucher_secret(&raw_pin)
}

#[tauri::command]
pub async fn calculate_consignor_settlement_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
    consignor_id: String,
    period_start: String,
    period_end: String,
    commission_percent: f64,
) -> Result<ConsignmentSettlement, String> {
    calculate_consignor_settlement(&pool, &consignor_id, &period_start, &period_end, commission_percent).await
}
