use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParkingTicket {
    pub id: String,
    pub ticket_code: String,
    pub vehicle_type: String,
    pub license_plate: Option<String>,
    pub entry_time: String,
    pub exit_time: Option<String>,
    pub duration_minutes: i32,
    pub total_fee: i32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelPumpReading {
    pub id: String,
    pub pump_id: String,
    pub fuel_type: String,
    pub initial_meter: f64,
    pub final_meter: f64,
    pub volume_liters: f64,
    pub shift_id: Option<String>,
    pub recorded_at: String,
}

pub fn calculate_parking_tariff(
    vehicle_type: &str,
    duration_minutes: i32,
    is_lost_ticket: bool,
) -> i32 {
    if is_lost_ticket {
        return match vehicle_type {
            "motorcycle" => 25000,
            "car" => 50000,
            _ => 100000,
        };
    }

    if duration_minutes <= 15 {
        return 0; // Grace period 15 mins free
    }

    let hours = ((duration_minutes as f64) / 60.0).ceil() as i32;

    let (first_hour, hourly_rate, max_daily) = match vehicle_type {
        "motorcycle" => (3000, 2000, 20000),
        "car" => (5000, 4000, 50000),
        _ => (10000, 8000, 100000), // bus_truck
    };

    if hours <= 1 {
        first_hour
    } else {
        let total = first_hour + (hours - 1) * hourly_rate;
        total.min(max_daily)
    }
}

pub async fn issue_parking_ticket(
    pool: &Pool<Sqlite>,
    vehicle_type: &str,
    license_plate: Option<&str>,
) -> Result<ParkingTicket, String> {
    let id = Uuid::new_v4().to_string();
    let ticket_code = format!("PRK-{}", &id[..8].to_uppercase());
    let entry_time = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO parking_tickets (id, ticket_code, vehicle_type, license_plate, entry_time, status, created_at)
         VALUES (?, ?, ?, ?, ?, 'inside', ?)"
    )
    .bind(&id)
    .bind(&ticket_code)
    .bind(vehicle_type)
    .bind(license_plate)
    .bind(&entry_time)
    .bind(&entry_time)
    .execute(pool)
    .await
    .map_err(|e| format!("Gagal mencetak tiket parkir: {}", e))?;

    Ok(ParkingTicket {
        id,
        ticket_code,
        vehicle_type: vehicle_type.to_string(),
        license_plate: license_plate.map(|s| s.to_string()),
        entry_time,
        exit_time: None,
        duration_minutes: 0,
        total_fee: 0,
        status: "inside".to_string(),
    })
}

pub async fn process_parking_exit(
    pool: &Pool<Sqlite>,
    ticket_code: &str,
    is_lost_ticket: bool,
) -> Result<ParkingTicket, String> {
    let exit_time = chrono::Utc::now();
    let exit_time_str = exit_time.to_rfc3339();

    let row = sqlx::query("SELECT id, vehicle_type, license_plate, entry_time, status FROM parking_tickets WHERE ticket_code = ?")
        .bind(ticket_code)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

    let (id, vehicle_type, license_plate, entry_time_str) = match row {
        Some(r) => {
            let st: String = r.get("status");
            if st != "inside" {
                return Err(format!("TICKET_ALREADY_CLOSED: Tiket parkir '{}' berstatus '{}'", ticket_code, st));
            }
            (
                r.get::<String, _>("id"),
                r.get::<String, _>("vehicle_type"),
                r.get::<Option<String>, _>("license_plate"),
                r.get::<String, _>("entry_time"),
            )
        }
        None => return Err(format!("TICKET_NOT_FOUND: Tiket parkir '{}' tidak ditemukan", ticket_code)),
    };

    let entry_time = chrono::DateTime::parse_from_rfc3339(&entry_time_str)
        .map_err(|_| "Invalid entry timestamp")?
        .with_timezone(&chrono::Utc);

    let duration_minutes = ((exit_time - entry_time).num_seconds() as f64 / 60.0).ceil() as i32;
    let total_fee = calculate_parking_tariff(&vehicle_type, duration_minutes, is_lost_ticket);
    let final_status = if is_lost_ticket { "lost_ticket" } else { "paid_exited" };

    sqlx::query(
        "UPDATE parking_tickets SET exit_time = ?, duration_minutes = ?, total_fee = ?, status = ? WHERE id = ?"
    )
    .bind(&exit_time_str)
    .bind(duration_minutes)
    .bind(total_fee)
    .bind(final_status)
    .bind(&id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(ParkingTicket {
        id,
        ticket_code: ticket_code.to_string(),
        vehicle_type,
        license_plate,
        entry_time: entry_time_str,
        exit_time: Some(exit_time_str),
        duration_minutes,
        total_fee,
        status: final_status.to_string(),
    })
}

pub async fn record_fuel_pump_reading(
    pool: &Pool<Sqlite>,
    pump_id: &str,
    fuel_type: &str,
    initial_meter: f64,
    final_meter: f64,
    shift_id: Option<&str>,
) -> Result<FuelPumpReading, String> {
    if final_meter < initial_meter {
        return Err("INVALID_METER_READING: Final meter tidak boleh lebih kecil dari initial meter".into());
    }

    let volume_liters = final_meter - initial_meter;
    let id = Uuid::new_v4().to_string();
    let recorded_at = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO fuel_pump_readings (id, pump_id, fuel_type, initial_meter, final_meter, volume_liters, shift_id, recorded_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(pump_id)
    .bind(fuel_type)
    .bind(initial_meter)
    .bind(final_meter)
    .bind(volume_liters)
    .bind(shift_id)
    .bind(&recorded_at)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(FuelPumpReading {
        id,
        pump_id: pump_id.to_string(),
        fuel_type: fuel_type.to_string(),
        initial_meter,
        final_meter,
        volume_liters,
        shift_id: shift_id.map(|s| s.to_string()),
        recorded_at,
    })
}

// Tauri commands
#[tauri::command]
pub async fn issue_parking_ticket_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
    vehicle_type: String,
    license_plate: Option<String>,
) -> Result<ParkingTicket, String> {
    issue_parking_ticket(&pool, &vehicle_type, license_plate.as_deref()).await
}

#[tauri::command]
pub async fn process_parking_exit_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
    ticket_code: String,
    is_lost_ticket: bool,
) -> Result<ParkingTicket, String> {
    process_parking_exit(&pool, &ticket_code, is_lost_ticket).await
}

#[tauri::command]
pub async fn record_fuel_pump_reading_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
    pump_id: String,
    fuel_type: String,
    initial_meter: f64,
    final_meter: f64,
    shift_id: Option<String>,
) -> Result<FuelPumpReading, String> {
    record_fuel_pump_reading(&pool, &pump_id, &fuel_type, initial_meter, final_meter, shift_id.as_deref()).await
}
