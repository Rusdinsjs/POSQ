use sqlx::sqlite::{SqlitePoolOptions, SqliteConnectOptions};
use sqlx::SqlitePool;
use std::str::FromStr;
use std::path::PathBuf;

/// Resolve the on-disk SQLite file path used by the local operational DB.
/// Mirrors the resolution in `establish_connection` so backup/restore can target the same file.
pub fn database_file_path() -> PathBuf {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        if let Some(path) = url.strip_prefix("sqlite://") {
            return PathBuf::from(path);
        }
    }
    let mut path = dirs::data_dir().expect("Could not find local app data directory");
    path.push("POSQ");
    std::fs::create_dir_all(&path).ok();
    path.push("posq.db");
    path
}

pub async fn establish_connection() -> Result<SqlitePool, String> {
    // Determine the database URL. Check environment first, otherwise fall back to local file.
    let database_url = if let Ok(url) = std::env::var("DATABASE_URL") {
        url
    } else {
        let mut path = dirs::data_dir().ok_or_else(|| "Could not find local app data directory".to_string())?;
        path.push("POSQ");
        std::fs::create_dir_all(&path).map_err(|e| format!("Failed to create POSQ data directory: {}", e))?;
        path.push("posq.db");
        format!("sqlite://{}", path.to_string_lossy())
    };

    let connection_options = SqliteConnectOptions::from_str(&database_url)
        .map_err(|e| format!("Invalid connection URL: {}", e))?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_millis(5000));

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connection_options)
        .await
        .map_err(|e| format!("Failed to connect to SQLite: {}", e))?;

    run_capability_migrations(&pool).await?;
    run_sync_migrations(&pool).await?;
    run_inventory_migrations(&pool).await?;

    Ok(pool)
}

pub async fn run_inventory_migrations(pool: &SqlitePool) -> Result<(), String> {
    sqlx::query("ALTER TABLE products ADD COLUMN erp_item_id TEXT;").execute(pool).await.ok();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS product_recipes (
            id TEXT PRIMARY KEY,
            product_id TEXT NOT NULL,
            ingredient_id TEXT NOT NULL,
            quantity REAL NOT NULL DEFAULT 1.0,
            unit TEXT NOT NULL DEFAULT 'pcs',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
            FOREIGN KEY (ingredient_id) REFERENCES products(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_product_recipes_prod 
            ON product_recipes(product_id);

        CREATE TABLE IF NOT EXISTS stock_movements (
            id TEXT PRIMARY KEY,
            merchant_id TEXT NOT NULL,
            outlet_id TEXT NOT NULL,
            product_id TEXT NOT NULL,
            movement_type TEXT NOT NULL,
            qty_delta REAL NOT NULL,
            reason TEXT,
            reference_number TEXT,
            erp_synced INTEGER NOT NULL DEFAULT 0,
            created_by TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_stock_movements_prod 
            ON stock_movements(product_id, created_at);
        "#
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to run inventory migrations: {}", e))?;

    Ok(())
}

pub async fn run_capability_migrations(pool: &SqlitePool) -> Result<(), String> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS outlet_capabilities (
            outlet_id TEXT NOT NULL,
            capability TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            granted_at TEXT NOT NULL DEFAULT (datetime('now')),
            granted_by TEXT,
            PRIMARY KEY (outlet_id, capability),
            FOREIGN KEY (outlet_id) REFERENCES outlets(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_outlet_caps_outlet 
            ON outlet_capabilities(outlet_id);
        "#
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to run capability migrations: {}", e))?;

    Ok(())
}

pub async fn run_sync_migrations(pool: &SqlitePool) -> Result<(), String> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sync_outbox (
            id TEXT PRIMARY KEY,
            event_id TEXT NOT NULL UNIQUE,
            event_type TEXT NOT NULL,
            aggregate_type TEXT NOT NULL,
            aggregate_id TEXT NOT NULL,
            aggregate_version INTEGER NOT NULL DEFAULT 1,
            schema_version INTEGER NOT NULL DEFAULT 1,
            merchant_id TEXT NOT NULL,
            outlet_id TEXT NOT NULL,
            device_id TEXT NOT NULL,
            actor_id TEXT,
            payload_json TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            retry_count INTEGER NOT NULL DEFAULT 0,
            last_error TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            pushed_at TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_sync_outbox_status_created 
            ON sync_outbox(status, created_at);

        CREATE INDEX IF NOT EXISTS idx_sync_outbox_event_id 
            ON sync_outbox(event_id);

        CREATE TABLE IF NOT EXISTS sync_inbox (
            id TEXT PRIMARY KEY,
            event_id TEXT NOT NULL UNIQUE,
            event_type TEXT NOT NULL,
            aggregate_type TEXT NOT NULL,
            aggregate_id TEXT NOT NULL,
            payload_json TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'applied',
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_sync_inbox_event_id 
            ON sync_inbox(event_id);
        "#
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to run sync migrations: {}", e))?;

    Ok(())
}

/// I-7: Local SQLite health check (LOCAL_POSTGRESQL_STRATEGY §8).
/// Verifies connectivity, required tables, schema marker, and free disk space.
/// Returns (state, detail) where state is one of OK / WARNING / ACTION_REQUIRED / BLOCKED.
pub async fn check_db_health(pool: &SqlitePool) -> (String, String) {
    // 1. Connectivity
    if sqlx::query("SELECT 1").execute(pool).await.is_err() {
        return ("BLOCKED".to_string(), "Cannot connect to local database".to_string());
    }

    // 2. Required operational tables
    let required = [
        "users", "roles", "products", "inventory_items", "orders",
        "order_items", "payments", "shifts", "audit_logs", "system_settings",
    ];
    let mut missing = Vec::new();
    for t in required {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name = ?"
        )
        .bind(t)
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        if exists == 0 {
            missing.push(t);
        }
    }
    if !missing.is_empty() {
        return (
            "ACTION_REQUIRED".to_string(),
            format!("Missing required tables: {}", missing.join(", ")),
        );
    }

    // 3. Migration marker present
    let migrated: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations'"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    if migrated == 0 {
        return (
            "WARNING".to_string(),
            "Schema migration table not found; schema version unknown".to_string(),
        );
    }

    // 4. Disk space on the volume holding the DB file.
    // Best-effort: if we cannot determine free space, do not fail the health check.
    let path = database_file_path();
    if let Some(parent) = path.parent() {
        if let Ok(avail) = free_space_bytes(parent) {
            if avail < 50 * 1024 * 1024 {
                return (
                    "WARNING".to_string(),
                    format!("Low disk space: {} MB free", avail / (1024 * 1024)),
                );
            }
        }
    }

    ("OK".to_string(), "Local database is healthy".to_string())
}

/// Best-effort free disk space in bytes for the volume containing `path`.
/// Uses `GetDiskFreeSpaceExW` on Windows, and a safe fallback elsewhere.
#[cfg(target_os = "windows")]
fn free_space_bytes(path: &std::path::Path) -> Result<u64, std::io::Error> {
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use winapi::um::fileapi::GetDiskFreeSpaceExW;
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let mut free: u64 = 0;
    unsafe {
        if GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut free as *mut u64 as *mut _,
            ptr::null_mut(),
            ptr::null_mut(),
        ) == 0
        {
            return Err(std::io::Error::last_os_error());
        }
    }
    Ok(free)
}

#[cfg(not(target_os = "windows"))]
fn free_space_bytes(_path: &std::path::Path) -> Result<u64, std::io::Error> {
    // Fallback: assume ample space; refine with a crate (e.g. sysinfo) cross-platform later.
    Ok(1024 * 1024 * 1024)
}

pub fn get_numeric_as_f64(row: &sqlx::sqlite::SqliteRow, column: &str) -> f64 {
    use sqlx::Row;
    row.try_get::<f64, _>(column)
        .or_else(|_| row.try_get::<i64, _>(column).map(|i| i as f64))
        .unwrap_or(0.0)
}
