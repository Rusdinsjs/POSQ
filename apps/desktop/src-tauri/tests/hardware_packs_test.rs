use sqlx::SqlitePool;

#[tokio::test]
async fn test_parking_progressive_tariff_calculation() {
    // Motorcycle: 3000 1st hour, 2000 per subsequent hour, max 20000
    let m_grace = calculate_parking_tariff("motorcycle", 10, false);
    let m_1h = calculate_parking_tariff("motorcycle", 45, false);
    let m_3h = calculate_parking_tariff("motorcycle", 150, false);
    let m_lost = calculate_parking_tariff("motorcycle", 60, true);

    assert_eq!(m_grace, 0, "Grace period <= 15 mins should be free");
    assert_eq!(m_1h, 3000, "1st hour motorcycle rate = 3000");
    assert_eq!(m_3h, 7000, "3 hours motorcycle rate = 3000 + 2000*2 = 7000");
    assert_eq!(m_lost, 25000, "Lost ticket motorcycle fee = 25000");

    // Car: 5000 1st hour, 4000 per subsequent hour, max 50000
    let c_1h = calculate_parking_tariff("car", 30, false);
    let c_5h = calculate_parking_tariff("car", 270, false);
    let c_lost = calculate_parking_tariff("car", 60, true);

    assert_eq!(c_1h, 5000);
    assert_eq!(c_5h, 21000); // 5000 + 4000*4 = 21000
    assert_eq!(c_lost, 50000);
}

fn calculate_parking_tariff(vehicle_type: &str, duration_minutes: i32, is_lost_ticket: bool) -> i32 {
    if is_lost_ticket {
        return match vehicle_type {
            "motorcycle" => 25000,
            "car" => 50000,
            _ => 100000,
        };
    }

    if duration_minutes <= 15 {
        return 0;
    }

    let hours = ((duration_minutes as f64) / 60.0).ceil() as i32;

    let (first_hour, hourly_rate, max_daily) = match vehicle_type {
        "motorcycle" => (3000, 2000, 20000),
        "car" => (5000, 4000, 50000),
        _ => (10000, 8000, 100000),
    };

    if hours <= 1 {
        first_hour
    } else {
        let total = first_hour + (hours - 1) * hourly_rate;
        total.min(max_daily)
    }
}

#[tokio::test]
async fn test_parking_ticket_database_workflow() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        "CREATE TABLE parking_tickets (
            id TEXT PRIMARY KEY,
            ticket_code TEXT UNIQUE NOT NULL,
            vehicle_type TEXT NOT NULL,
            license_plate TEXT,
            entry_time TEXT NOT NULL,
            exit_time TEXT,
            duration_minutes INTEGER NOT NULL DEFAULT 0,
            total_fee INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'inside',
            created_at TEXT NOT NULL
        );"
    )
    .execute(&pool)
    .await
    .unwrap();

    let now = chrono::Utc::now().to_rfc3339();

    // Entry gate
    sqlx::query(
        "INSERT INTO parking_tickets (id, ticket_code, vehicle_type, license_plate, entry_time, status, created_at)
         VALUES ('prk_1', 'PRK-8888', 'car', 'B 1234 ABC', ?, 'inside', ?)"
    )
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    let status: String = sqlx::query_scalar("SELECT status FROM parking_tickets WHERE ticket_code = 'PRK-8888'").fetch_one(&pool).await.unwrap();
    assert_eq!(status, "inside");

    // Exit gate
    sqlx::query(
        "UPDATE parking_tickets SET status = 'paid_exited', exit_time = ?, duration_minutes = 120, total_fee = 9000 WHERE ticket_code = 'PRK-8888'"
    )
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    let final_status: String = sqlx::query_scalar("SELECT status FROM parking_tickets WHERE ticket_code = 'PRK-8888'").fetch_one(&pool).await.unwrap();
    let total_fee: i32 = sqlx::query_scalar("SELECT total_fee FROM parking_tickets WHERE ticket_code = 'PRK-8888'").fetch_one(&pool).await.unwrap();

    assert_eq!(final_status, "paid_exited");
    assert_eq!(total_fee, 9000);
}

#[tokio::test]
async fn test_fuel_pump_meter_reconciliation() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        "CREATE TABLE fuel_pump_readings (
            id TEXT PRIMARY KEY,
            pump_id TEXT NOT NULL,
            fuel_type TEXT NOT NULL,
            initial_meter REAL NOT NULL,
            final_meter REAL NOT NULL,
            volume_liters REAL NOT NULL,
            shift_id TEXT,
            recorded_at TEXT NOT NULL
        );"
    )
    .execute(&pool)
    .await
    .unwrap();

    let now = chrono::Utc::now().to_rfc3339();

    let initial = 125000.0;
    let final_m = 125450.5;
    let volume = final_m - initial;

    sqlx::query(
        "INSERT INTO fuel_pump_readings (id, pump_id, fuel_type, initial_meter, final_meter, volume_liters, recorded_at)
         VALUES ('fp_1', 'PUMP-01', 'pertalite', ?, ?, ?, ?)"
    )
    .bind(initial)
    .bind(final_m)
    .bind(volume)
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    let volume_rec: f64 = sqlx::query_scalar("SELECT volume_liters FROM fuel_pump_readings WHERE id = 'fp_1'").fetch_one(&pool).await.unwrap();
    assert_eq!(volume_rec, 450.5);
}
