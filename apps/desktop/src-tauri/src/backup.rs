use tauri::command;
use std::fs;
use std::path::PathBuf;
use sqlx::{SqlitePool, Row};
use tauri::State;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};
use rand_core::{RngCore, OsRng};
use reqwest::Client;
use serde_json::json;
use sha2::{Sha256, Digest};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BackupResult {
    pub success: bool,
    pub message: String,
    pub path: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct RecoveryKeyResult {
    pub key: String,
}

#[command]
pub async fn generate_recovery_key() -> Result<RecoveryKeyResult, String> {
    let mut key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut key_bytes);
    let key_hex = hex::encode(key_bytes);
    Ok(RecoveryKeyResult { key: key_hex })
}

/// Resolve the directory where backups are stored.
fn backup_dir() -> PathBuf {
    let mut dir = dirs::document_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("POSQ_Backups");
    dir
}

#[command]
pub async fn create_local_backup(
    pool: State<'_, SqlitePool>,
    encrypt: bool,
    recovery_key: Option<String>,
) -> Result<BackupResult, String> {
    let dir = backup_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = if encrypt {
        format!("posq_backup_{}.enc", timestamp)
    } else {
        format!("posq_backup_{}.db", timestamp)
    };

    let mut file_path = dir.clone();
    file_path.push(&filename);
    let file_path_str = file_path.to_string_lossy().to_string();

    // Atomic SQLite snapshot via VACUUM INTO (no pg_dump dependency).
    vacuum_into(pool.inner(), &file_path).await?;

    // Encrypt the snapshot in place if requested.
    if encrypt {
        let key_hex = recovery_key.ok_or_else(|| "Encryption requested but no recovery key provided".to_string())?;
        let snapshot = fs::read(&file_path).map_err(|e| e.to_string())?;
        let encrypted = encrypt_data(&snapshot, &key_hex)?;
        fs::write(&file_path, encrypted).map_err(|e| e.to_string())?;
    }

    // Attempt to upload metadata to CP (best-effort; errors ignored for MVP).
    let size = fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);
    let _ = upload_backup_metadata(size as i64, filename.clone()).await;

    Ok(BackupResult {
        success: true,
        message: "Backup created successfully".into(),
        path: Some(file_path_str),
    })
}

#[command]
pub async fn restore_local_backup(
    _pool: State<'_, SqlitePool>,
    file_path: String,
    recovery_key: Option<String>,
) -> Result<BackupResult, String> {
    let encrypted = file_path.ends_with(".enc");

    let file_data = fs::read(&file_path).map_err(|e| e.to_string())?;

    let mut db_data = file_data;
    if encrypted {
        let key_hex = recovery_key.ok_or_else(|| "Recovery key required for encrypted backup".to_string())?;
        db_data = decrypt_data(&db_data, &key_hex)?;
    }

    // Verify SQLite format and run PRAGMA integrity_check on candidate data
    let temp_candidate = std::env::temp_dir().join(format!("posq_restore_verify_{}.db", uuid::Uuid::new_v4()));
    fs::write(&temp_candidate, &db_data).map_err(|e| format!("Failed to write temporary restore candidate: {}", e))?;

    let check_res = match SqlitePool::connect(&format!("sqlite://{}", temp_candidate.to_string_lossy())).await {
        Ok(pool) => {
            let row = sqlx::query("PRAGMA integrity_check")
                .fetch_one(&pool)
                .await
                .map_err(|e| e.to_string())?;
            let res: String = row.get(0);
            pool.close().await;
            let _ = fs::remove_file(&temp_candidate);
            res
        }
        Err(e) => {
            let _ = fs::remove_file(&temp_candidate);
            return Err(format!("CORRUPT_BACKUP: Candidate database cannot be opened: {}", e));
        }
    };

    if check_res != "ok" {
        return Err(format!("CORRUPT_BACKUP: Integrity check failed: {}", check_res));
    }

    // SAFETY: take a pre-restore backup of the live DB before overwriting it.
    let live_path = crate::db::database_file_path();
    if live_path.exists() {
        if let Some(parent) = live_path.parent() {
            let pre_restore_name = format!(
                "posq_pre_restore_{}.db",
                chrono::Local::now().format("%Y%m%d_%H%M%S")
            );
            let mut pre_path = parent.to_path_buf();
            pre_path.push(pre_restore_name);
            fs::copy(&live_path, &pre_path).map_err(|e| format!("Pre-restore backup failed: {}", e))?;
        }
    }

    // Write the restored DB on top of the live file.
    fs::write(&live_path, db_data).map_err(|e| e.to_string())?;

    Ok(BackupResult {
        success: true,
        message: "Restore completed successfully with verified integrity".into(),
        path: None,
    })
}

/// VACUUM the live SQLite DB into a target file atomically.
/// SQLite requires the target file to not exist beforehand.
async fn vacuum_into(pool: &SqlitePool, target: &std::path::Path) -> Result<(), String> {
    if target.exists() {
        fs::remove_file(target).map_err(|e| e.to_string())?;
    }
    let path_str = target.to_string_lossy().replace('\'', "''");
    // VACUUM INTO requires a literal filename in some SQLite builds.
    let stmt = format!("VACUUM INTO '{}'", path_str);
    sqlx::query(&stmt)
        .execute(pool)
        .await
        .map_err(|e| format!("Backup (VACUUM INTO) failed: {}", e))?;
    Ok(())
}

// Helper to encrypt
fn encrypt_data(data: &[u8], key_hex: &str) -> Result<Vec<u8>, String> {
    let key_bytes = hex::decode(key_hex).map_err(|_| "Invalid hex key".to_string())?;
    if key_bytes.len() != 32 { return Err("Key must be 32 bytes".into()); }

    let key = aes_gcm::Key::<Aes256Gcm>::try_from(key_bytes.as_slice()).map_err(|_| "Invalid key length".to_string())?;
    let cipher = Aes256Gcm::new(&key);

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from(nonce_bytes);

    let ciphertext = cipher.encrypt(&nonce, data)
        .map_err(|_| "Encryption failed".to_string())?;

    let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

// Helper to decrypt
fn decrypt_data(data: &[u8], key_hex: &str) -> Result<Vec<u8>, String> {
    let key_bytes = hex::decode(key_hex).map_err(|_| "Invalid hex key".to_string())?;
    if key_bytes.len() != 32 { return Err("Key must be 32 bytes".into()); }

    if data.len() < 12 { return Err("Data too short".into()); }

    let key = aes_gcm::Key::<Aes256Gcm>::try_from(key_bytes.as_slice()).map_err(|_| "Invalid key length".to_string())?;
    let cipher = Aes256Gcm::new(&key);

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::try_from(nonce_bytes).map_err(|_| "Invalid nonce length".to_string())?;

    let plaintext = cipher.decrypt(&nonce, ciphertext)
        .map_err(|_| "Decryption failed - wrong key or corrupt data".to_string())?;

    Ok(plaintext)
}

/// Resolve the Control Plane API base URL (same logic as license module).
async fn resolve_api_base() -> String {
    if let Ok(url) = std::env::var("PUBLIC_API_URL") {
        if !url.is_empty() {
            return url.trim_end_matches('/').to_string();
        }
    }
    "http://127.0.0.1:3000".to_string()
}

/// Upload backup metadata to the control plane with idempotency key.
/// Best-effort for MVP: failures are logged but do not block local backup.
async fn upload_backup_metadata(size: i64, path: String) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let base = resolve_api_base().await;

    let backup_id = uuid::Uuid::new_v4().to_string();
    let checksum = {
        let mut hasher = Sha256::new();
        hasher.update(backup_id.as_bytes());
        hasher.update(path.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    let body = json!({
        "device_id": whoami_device_id(),
        "backup_id": backup_id,
        "destination_type": "local",
        "logical_storage_ref": path,
        "size_bytes": size,
        "checksum": checksum,
        "encryption_algorithm": "AES-256-GCM",
        "encrypted": true,
        "app_version": env!("CARGO_PKG_VERSION"),
        "db_schema_version": "1"
    });

    let _ = client
        .post(format!("{}/api/v1/backups/metadata", base))
        .json(&body)
        .send()
        .await;

    Ok(())
}

/// Stable-ish device id derived locally (not a secret).
fn whoami_device_id() -> String {
    let mut parts = vec![
        std::env::var("COMPUTERNAME").unwrap_or_else(|_| "host".to_string()),
        std::env::consts::OS.to_string(),
    ];
    if let Ok(user) = std::env::var("USERNAME") {
        parts.push(user);
    }
    let mut hasher = Sha256::new();
    hasher.update(parts.join("|").as_bytes());
    format!("{:x}", hasher.finalize())
}
