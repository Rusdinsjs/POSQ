use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffResource {
    pub id: String,
    pub name: String,
    pub role: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appointment {
    pub id: String,
    pub customer_id: Option<String>,
    pub staff_id: String,
    pub service_product_id: String,
    pub scheduled_at: String,
    pub duration_minutes: i32,
    pub status: String,
    pub deposit_amount: i32,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAppointmentPayload {
    pub customer_id: Option<String>,
    pub staff_id: String,
    pub service_product_id: String,
    pub scheduled_at: String,
    pub duration_minutes: i32,
    pub deposit_amount: i32,
    pub notes: Option<String>,
}

pub async fn create_appointment(
    pool: &Pool<Sqlite>,
    payload: CreateAppointmentPayload,
) -> Result<Appointment, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO appointments (id, customer_id, staff_id, service_product_id, scheduled_at, duration_minutes, status, deposit_amount, notes, created_at)
         VALUES (?, ?, ?, ?, ?, ?, 'scheduled', ?, ?, ?)"
    )
    .bind(&id)
    .bind(&payload.customer_id)
    .bind(&payload.staff_id)
    .bind(&payload.service_product_id)
    .bind(&payload.scheduled_at)
    .bind(payload.duration_minutes)
    .bind(payload.deposit_amount)
    .bind(&payload.notes)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("Gagal membuat janji temu: {}", e))?;

    Ok(Appointment {
        id,
        customer_id: payload.customer_id,
        staff_id: payload.staff_id,
        service_product_id: payload.service_product_id,
        scheduled_at: payload.scheduled_at,
        duration_minutes: payload.duration_minutes,
        status: "scheduled".to_string(),
        deposit_amount: payload.deposit_amount,
        notes: payload.notes,
        created_at: now,
    })
}

pub async fn update_appointment_status(
    pool: &Pool<Sqlite>,
    appointment_id: &str,
    status: &str,
) -> Result<(), String> {
    let allowed = ["scheduled", "checked_in", "in_service", "completed", "cancelled", "no_show"];
    if !allowed.contains(&status) {
        return Err(format!("INVALID_STATUS: Status janji temu '{}' tidak valid", status));
    }

    sqlx::query("UPDATE appointments SET status = ? WHERE id = ?")
        .bind(status)
        .bind(appointment_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// Tauri commands
#[tauri::command]
pub async fn create_appointment_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
    payload: CreateAppointmentPayload,
) -> Result<Appointment, String> {
    create_appointment(&pool, payload).await
}

#[tauri::command]
pub async fn list_appointments_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<Vec<Appointment>, String> {
    let rows = sqlx::query("SELECT id, customer_id, staff_id, service_product_id, scheduled_at, duration_minutes, status, deposit_amount, notes, created_at FROM appointments ORDER BY scheduled_at ASC")
        .fetch_all(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    let appts = rows.into_iter().map(|r| Appointment {
        id: r.get("id"),
        customer_id: r.get("customer_id"),
        staff_id: r.get("staff_id"),
        service_product_id: r.get("service_product_id"),
        scheduled_at: r.get("scheduled_at"),
        duration_minutes: r.get("duration_minutes"),
        status: r.get("status"),
        deposit_amount: r.get("deposit_amount"),
        notes: r.get("notes"),
        created_at: r.get("created_at"),
    }).collect();

    Ok(appts)
}
