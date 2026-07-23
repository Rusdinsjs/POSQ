use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RentalContract {
    pub id: String,
    pub asset_id: String,
    pub customer_id: Option<String>,
    pub deposit_amount: i32,
    pub start_at: String,
    pub due_at: String,
    pub returned_at: Option<String>,
    pub late_fee: i32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembershipSubscription {
    pub id: String,
    pub customer_id: String,
    pub plan_name: String,
    pub remaining_credits: i32,
    pub valid_until: String,
    pub active: bool,
}

pub async fn checkout_rental_asset(
    pool: &Pool<Sqlite>,
    asset_id: &str,
    customer_id: Option<&str>,
    deposit_amount: i32,
    duration_hours: i64,
) -> Result<RentalContract, String> {
    let id = Uuid::new_v4().to_string();
    let start_time = chrono::Utc::now();
    let due_time = start_time + chrono::Duration::hours(duration_hours);
    let start_at = start_time.to_rfc3339();
    let due_at = due_time.to_rfc3339();
    let created_at = start_at.clone();

    sqlx::query(
        "INSERT INTO rental_contracts (id, asset_id, customer_id, deposit_amount, start_at, due_at, status, created_at)
         VALUES (?, ?, ?, ?, ?, ?, 'checked_out', ?)"
    )
    .bind(&id)
    .bind(asset_id)
    .bind(customer_id)
    .bind(deposit_amount)
    .bind(&start_at)
    .bind(&due_at)
    .bind(&created_at)
    .execute(pool)
    .await
    .map_err(|e| format!("Gagal menyewa aset: {}", e))?;

    Ok(RentalContract {
        id,
        asset_id: asset_id.to_string(),
        customer_id: customer_id.map(|s| s.to_string()),
        deposit_amount,
        start_at,
        due_at,
        returned_at: None,
        late_fee: 0,
        status: "checked_out".to_string(),
    })
}

pub async fn return_rental_asset(
    pool: &Pool<Sqlite>,
    contract_id: &str,
    hourly_late_rate: i32,
) -> Result<i32, String> {
    let now = chrono::Utc::now();
    let now_str = now.to_rfc3339();

    let row = sqlx::query("SELECT due_at, status FROM rental_contracts WHERE id = ?")
        .bind(contract_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

    let due_at_str = match row {
        Some(r) => {
            let st: String = r.get("status");
            if st == "returned" {
                return Err("ASSET_ALREADY_RETURNED: Kontrak sewa sudah dikembalikan sebelumnya".into());
            }
            r.get::<String, _>("due_at")
        }
        None => return Err(format!("CONTRACT_NOT_FOUND: Kontrak sewa '{}' tidak ditemukan", contract_id)),
    };

    let due_at = chrono::DateTime::parse_from_rfc3339(&due_at_str)
        .map_err(|_| "Invalid timestamp format")?
        .with_timezone(&chrono::Utc);

    let late_fee = if now > due_at {
        let diff_hours = (now - due_at).num_hours() + 1;
        (diff_hours as i32) * hourly_late_rate
    } else {
        0
    };

    sqlx::query("UPDATE rental_contracts SET status = 'returned', returned_at = ?, late_fee = ? WHERE id = ?")
        .bind(&now_str)
        .bind(late_fee)
        .bind(contract_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(late_fee)
}

// Tauri commands
#[tauri::command]
pub async fn checkout_rental_asset_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
    asset_id: String,
    customer_id: Option<String>,
    deposit_amount: i32,
    duration_hours: i64,
) -> Result<RentalContract, String> {
    checkout_rental_asset(&pool, &asset_id, customer_id.as_deref(), deposit_amount, duration_hours).await
}

#[tauri::command]
pub async fn return_rental_asset_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
    contract_id: String,
    hourly_late_rate: i32,
) -> Result<i32, String> {
    return_rental_asset(&pool, &contract_id, hourly_late_rate).await
}
