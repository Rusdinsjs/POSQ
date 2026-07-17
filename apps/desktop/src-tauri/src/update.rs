use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::{command, State};
use ed25519_dalek::{VerifyingKey, Signature, Verifier};
use reqwest::Client;

#[derive(Serialize, Deserialize, Clone)]
pub struct UpdateMetadata {
    pub version: String,
    pub release_notes: String,
    pub download_url: String,
    pub signature: String,
    pub channel: String,
    pub os: String,
    pub sha256: String,
}

impl UpdateMetadata {
    /// Reconstruct the exact payload the control plane signs (services/update.rs),
    /// so the desktop can verify the server signature with UPDATE_PUBLIC_KEY.
    pub fn signed_payload_json(&self) -> String {
        serde_json::json!({
            "version": self.version,
            "channel": self.channel,
            "os": self.os,
            "sha256": self.sha256,
            "download_url": self.download_url,
        })
        .to_string()
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub success: bool,
    pub update_available: bool,
    pub metadata: Option<UpdateMetadata>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SafeMigrationResult {
    pub success: bool,
    pub message: String,
}

// Separate Public Key for Update Signature validation (DEC-054 Compliance)
const UPDATE_PUBLIC_KEY: [u8; 32] = [
    96, 145, 14, 76, 146, 62, 190, 168, 8, 174, 137, 137, 122, 140, 102, 75, 244, 40, 125, 147, 91, 
    121, 202, 75, 37, 152, 224, 121, 175, 154, 106, 90
];

#[command]
pub async fn check_update(channel: String) -> Result<UpdateCheckResult, String> {
    let current_version = env!("CARGO_PKG_VERSION");
    let base = resolve_update_api_base().await;

    // M10-001 Version check against the control plane (DEC-029: server signs metadata;
    // no private key is held in the desktop app).
    let url = format!(
        "{}/api/v1/updates/check?os={}&channel={}&version={}",
        base,
        std::env::consts::OS,
        urlencode(&channel),
        current_version
    );
    let response: serde_json::Value = Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to contact update server: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Invalid update response: {}", e))?;

    let update_available = response
        .get("update_available")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !update_available {
        return Ok(UpdateCheckResult {
            success: true,
            update_available: false,
            metadata: None,
            error: None,
        });
    }

    let metadata = UpdateMetadata {
        version: response.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        release_notes: response
            .get("release_notes")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        download_url: response
            .get("download_url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        signature: response
            .get("signature")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        channel: channel.clone(),
        os: std::env::consts::OS.to_string(),
        sha256: response
            .get("sha256")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    };

    Ok(UpdateCheckResult {
        success: true,
        update_available: true,
        metadata: Some(metadata),
        error: None,
    })
}

/// Resolve the Control Plane API base URL for update checks.
async fn resolve_update_api_base() -> String {
    if let Ok(url) = std::env::var("PUBLIC_API_URL") {
        if !url.is_empty() {
            return url.trim_end_matches('/').to_string();
        }
    }
    "http://127.0.0.1:3000".to_string()
}

#[command]
pub async fn validate_update(metadata: UpdateMetadata, signature: String) -> Result<bool, String> {
    // M10-002 Signed update validation using Ed25519 (Blocker 3 Fixed)
    let verifying_key = VerifyingKey::from_bytes(&UPDATE_PUBLIC_KEY)
        .map_err(|e| format!("Invalid public key configuration: {}", e))?;
    
    let sig_bytes = hex::decode(&signature)
        .map_err(|_| "Signature is not valid hex".to_string())?;
    
    let signature = Signature::from_slice(&sig_bytes)
        .map_err(|_| "Signature must be 64 bytes".to_string())?;
    
    let message = metadata.signed_payload_json();
    
    verifying_key.verify(message.as_bytes(), &signature)
        .map(|_| true)
        .map_err(|e| format!("Update signature validation failed: {}", e))
}

#[command]
pub async fn run_safe_migration(pool: State<'_, SqlitePool>) -> Result<SafeMigrationResult, String> {
    // M10-003 Migration backup gate
    println!("Starting safe migration. Taking pre-migration backup...");
    
    // We create an unencrypted backup for safety during migration
    let backup_res = crate::backup::create_local_backup(pool.clone(), false, None).await;
    
    if let Err(e) = backup_res {
        return Err(format!("MIGRATION ABORTED: Pre-migration backup failed: {}", e));
    }
    
    let backup_data = backup_res.unwrap();
    if !backup_data.success {
        return Err(format!("MIGRATION ABORTED: Pre-migration backup returned failure: {}", backup_data.message));
    }
 
    println!("Backup successful at {:?}. Running SQL migrations...", backup_data.path);

    // Run migrations
    match crate::migration::run_migrations(pool.inner()).await {
        Ok(_) => Ok(SafeMigrationResult {
            success: true,
            message: "Migration completed successfully.".into(),
        }),
        Err(e) => {
            // M10-004 Failed migration recovery
            let err_msg = format!("MIGRATION FAILED! The system may be in an unstable state. Please restore the backup from: {:?} \nError details: {}", backup_data.path, e);
            Err(err_msg)
        }
    }
}

/// Minimal percent-encoding for query parameter values (channel may contain safe chars only).
fn urlencode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}
