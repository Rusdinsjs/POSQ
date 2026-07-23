use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedUnit {
    pub id: String,
    pub product_id: String,
    pub serial_no: String,
    pub imei1: Option<String>,
    pub imei2: Option<String>,
    pub status: String,
    pub unit_cost: i32,
    pub warranty_months: i32,
    pub received_at: String,
    pub sold_at: Option<String>,
    pub order_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntakeSerialPayload {
    pub product_id: String,
    pub serial_no: String,
    pub imei1: Option<String>,
    pub imei2: Option<String>,
    pub unit_cost: i32,
    pub warranty_months: Option<i32>,
}

pub async fn register_serial_unit(
    pool: &Pool<Sqlite>,
    payload: IntakeSerialPayload,
) -> Result<SerializedUnit, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let warranty = payload.warranty_months.unwrap_or(12);

    sqlx::query(
        "INSERT INTO serialized_units (id, product_id, serial_no, imei1, imei2, status, unit_cost, warranty_months, received_at)
         VALUES (?, ?, ?, ?, ?, 'in_stock', ?, ?, ?)"
    )
    .bind(&id)
    .bind(&payload.product_id)
    .bind(&payload.serial_no)
    .bind(&payload.imei1)
    .bind(&payload.imei2)
    .bind(payload.unit_cost)
    .bind(warranty)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("Gagal mendaftarkan nomor seri: {}", e))?;

    Ok(SerializedUnit {
        id,
        product_id: payload.product_id,
        serial_no: payload.serial_no,
        imei1: payload.imei1,
        imei2: payload.imei2,
        status: "in_stock".to_string(),
        unit_cost: payload.unit_cost,
        warranty_months: warranty,
        received_at: now,
        sold_at: None,
        order_id: None,
    })
}

pub async fn mark_serial_sold(
    pool: &Pool<Sqlite>,
    serial_no: &str,
    order_id: &str,
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();

    let row = sqlx::query("SELECT status FROM serialized_units WHERE serial_no = ?")
        .bind(serial_no)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

    match row {
        Some(r) => {
            let status: String = r.get("status");
            if status != "in_stock" {
                return Err(format!("SERIAL_ALREADY_SOLD: Nomor seri '{}' berstatus '{}'", serial_no, status));
            }
        }
        None => return Err(format!("SERIAL_NOT_FOUND: Nomor seri '{}' tidak ditemukan di inventaris", serial_no)),
    }

    sqlx::query("UPDATE serialized_units SET status = 'sold', sold_at = ?, order_id = ? WHERE serial_no = ?")
        .bind(&now)
        .bind(order_id)
        .bind(serial_no)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// Tauri commands
#[tauri::command]
pub async fn register_serial_unit_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
    payload: IntakeSerialPayload,
) -> Result<SerializedUnit, String> {
    register_serial_unit(&pool, payload).await
}

#[tauri::command]
pub async fn list_available_serials_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
    product_id: String,
) -> Result<Vec<SerializedUnit>, String> {
    let rows = sqlx::query("SELECT id, product_id, serial_no, imei1, imei2, status, unit_cost, warranty_months, received_at, sold_at, order_id FROM serialized_units WHERE product_id = ? AND status = 'in_stock'")
        .bind(&product_id)
        .fetch_all(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    let units = rows.into_iter().map(|r| SerializedUnit {
        id: r.get("id"),
        product_id: r.get("product_id"),
        serial_no: r.get("serial_no"),
        imei1: r.get("imei1"),
        imei2: r.get("imei2"),
        status: r.get("status"),
        unit_cost: r.get("unit_cost"),
        warranty_months: r.get("warranty_months"),
        received_at: r.get("received_at"),
        sold_at: r.get("sold_at"),
        order_id: r.get("order_id"),
    }).collect();

    Ok(units)
}
