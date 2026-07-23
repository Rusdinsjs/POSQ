use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerAsset {
    pub id: String,
    pub customer_id: Option<String>,
    pub asset_type: String,
    pub brand_model: String,
    pub serial_no: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairTicket {
    pub id: String,
    pub ticket_number: String,
    pub asset_id: String,
    pub customer_id: Option<String>,
    pub problem_description: String,
    pub status: String,
    pub estimated_cost: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRepairTicketPayload {
    pub customer_id: Option<String>,
    pub asset_type: String,
    pub brand_model: String,
    pub serial_no: Option<String>,
    pub problem_description: String,
    pub estimated_cost: i32,
}

pub async fn create_repair_ticket(
    pool: &Pool<Sqlite>,
    payload: CreateRepairTicketPayload,
) -> Result<RepairTicket, String> {
    let asset_id = Uuid::new_v4().to_string();
    let ticket_id = Uuid::new_v4().to_string();
    let ticket_no = format!("RPR-{}", &ticket_id[..8].to_uppercase());
    let now = chrono::Utc::now().to_rfc3339();

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        "INSERT INTO customer_assets (id, customer_id, asset_type, brand_model, serial_no, created_at)
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&asset_id)
    .bind(&payload.customer_id)
    .bind(&payload.asset_type)
    .bind(&payload.brand_model)
    .bind(&payload.serial_no)
    .bind(&now)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        "INSERT INTO repair_tickets (id, ticket_number, asset_id, customer_id, problem_description, status, estimated_cost, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, 'received', ?, ?, ?)"
    )
    .bind(&ticket_id)
    .bind(&ticket_no)
    .bind(&asset_id)
    .bind(&payload.customer_id)
    .bind(&payload.problem_description)
    .bind(payload.estimated_cost)
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(RepairTicket {
        id: ticket_id,
        ticket_number: ticket_no,
        asset_id,
        customer_id: payload.customer_id,
        problem_description: payload.problem_description,
        status: "received".to_string(),
        estimated_cost: payload.estimated_cost,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub async fn update_repair_status(
    pool: &Pool<Sqlite>,
    ticket_id: &str,
    status: &str,
) -> Result<(), String> {
    let allowed = ["received", "diagnosing", "awaiting_approval", "approved", "repairing", "ready", "collected"];
    if !allowed.contains(&status) {
        return Err(format!("INVALID_STATUS: Status tiket servis '{}' tidak valid", status));
    }

    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("UPDATE repair_tickets SET status = ?, updated_at = ? WHERE id = ?")
        .bind(status)
        .bind(&now)
        .bind(ticket_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// Tauri commands
#[tauri::command]
pub async fn create_repair_ticket_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
    payload: CreateRepairTicketPayload,
) -> Result<RepairTicket, String> {
    create_repair_ticket(&pool, payload).await
}

#[tauri::command]
pub async fn list_repair_tickets_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<Vec<RepairTicket>, String> {
    let rows = sqlx::query("SELECT id, ticket_number, asset_id, customer_id, problem_description, status, estimated_cost, created_at, updated_at FROM repair_tickets ORDER BY created_at DESC")
        .fetch_all(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    let tickets = rows.into_iter().map(|r| RepairTicket {
        id: r.get("id"),
        ticket_number: r.get("ticket_number"),
        asset_id: r.get("asset_id"),
        customer_id: r.get("customer_id"),
        problem_description: r.get("problem_description"),
        status: r.get("status"),
        estimated_cost: r.get("estimated_cost"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }).collect();

    Ok(tickets)
}
