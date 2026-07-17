use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::command;
use chrono::{DateTime, Utc};
use ed25519_dalek::{VerifyingKey, Signature, Verifier};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LicenseToken {
    pub merchant_id: String,
    pub device_id: String,
    pub plan: String,
    pub entitlements: Vec<String>,
    pub issued_at: String,
    pub valid_until: String,
    pub grace_until: String,
    pub status: String,
    pub signature: String, // Raw server-issued JWT token (opaque to desktop)
}

impl LicenseToken {
    /// Build a UI-friendly token view from a verified server JWT.
    fn from_jwt(raw: &str, claims: &LicenseClaims) -> Self {
        let now = Utc::now();
        let status = if now.timestamp() > claims.grace_until {
            "restricted_expired".to_string()
        } else if now.timestamp() > claims.exp {
            "grace".to_string()
        } else {
            "active".to_string()
        };
        LicenseToken {
            merchant_id: claims.merchant_id.clone(),
            device_id: claims.device_id.clone(),
            plan: claims.runtime_mode.clone(),
            entitlements: vec![],
            issued_at: DateTime::from_timestamp(claims.iat as i64, 0)
                .map(|d| d.to_rfc3339())
                .unwrap_or_default(),
            valid_until: DateTime::from_timestamp(claims.exp as i64, 0)
                .map(|d| d.to_rfc3339())
                .unwrap_or_default(),
            grace_until: DateTime::from_timestamp(claims.grace_until as i64, 0)
                .map(|d| d.to_rfc3339())
                .unwrap_or_default(),
            status,
            signature: raw.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LicenseStateResult {
    pub mode: String, // "Active", "Grace", "RestrictedExpired", "Unlicensed", "SuspiciousTime"
    pub token: Option<LicenseToken>,
    pub error: Option<String>,
}

fn get_license_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("POSQ");
    fs::create_dir_all(&path).unwrap_or(());
    path.push("license.json");
    path
}

fn get_last_seen_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("POSQ");
    fs::create_dir_all(&path).unwrap_or(());
    path.push("last_seen_time.txt");
    path
}

// Public key of the Control Plane for Ed25519 JWT verification (public only; DEC-029).
const SERVER_PUBLIC_KEY: [u8; 32] = [
    138, 165, 194, 189, 11, 14, 75, 103, 100, 102, 177, 229, 105, 66, 70, 127, 43, 47, 54, 4, 36,
    253, 91, 154, 207, 20, 179, 236, 159, 190, 151, 128
];

// Claims we extract from the server-issued Ed25519 JWT (ADR-0011).
#[derive(Serialize, Deserialize)]
struct LicenseClaims {
    merchant_id: String,
    device_id: String,
    runtime_mode: String,
    iat: i64,
    nbf: i64,
    exp: i64,
    grace_until: i64,
}

#[derive(Serialize, Deserialize)]
struct ActivationChallengeResponse {
    challenge_id: String,
    challenge: String,
    expires_at: String,
}

#[derive(Serialize, Deserialize)]
struct ActivateResponse {
    device_id: String,
    license_token: String,
    server_time: String,
    runtime_mode: String,
}

#[derive(Serialize, Deserialize)]
struct RefreshResponse {
    license_token: String,
    server_time: String,
    runtime_mode: String,
}

/// Resolve the Control Plane API base URL.
/// Priority: PUBLIC_API_URL env, then cp_api_url system setting, then local dev default.
async fn resolve_api_base(pool: Option<&sqlx::SqlitePool>) -> String {
    if let Ok(url) = std::env::var("PUBLIC_API_URL") {
        if !url.is_empty() {
            return url.trim_end_matches('/').to_string();
        }
    }
    if let Some(pool) = pool {
        if let Ok(rows) = sqlx::query("SELECT value FROM system_settings WHERE key = 'cp_api_url'")
            .fetch_optional(pool)
            .await
        {
            if let Some(row) = rows {
                use sqlx::Row;
                let v: String = row.try_get("value").unwrap_or_default();
                if !v.is_empty() {
                    return v.trim_end_matches('/').to_string();
                }
            }
        }
    }
    "http://127.0.0.1:3000".to_string()
}

/// Verify the Ed25519 (EdDSA) signature of a server-issued JWT and return its claims.
/// The token format is `header_b64.payload_b64.signature_b64` (base64url-safe, no padding).
fn verify_jwt(raw: &str) -> Result<LicenseClaims, String> {
    let parts: Vec<&str> = raw.split('.').collect();
    if parts.len() != 3 {
        return Err("Malformed license token".into());
    }
    let signing_input = format!("{}.{}", parts[0], parts[1]);
    let sig_bytes = BASE64
        .decode(parts[2])
        .map_err(|_| "Invalid signature encoding".to_string())?;
    let signature = Signature::from_slice(&sig_bytes)
        .map_err(|_| "Signature must be 64 bytes".to_string())?;

    let verifying_key = VerifyingKey::from_bytes(&SERVER_PUBLIC_KEY)
        .map_err(|e| format!("Invalid public key configuration: {}", e))?;

    verifying_key
        .verify(signing_input.as_bytes(), &signature)
        .map_err(|_| "License token signature is invalid. TAMPERED!".to_string())?;

    let payload_bytes = BASE64
        .decode(parts[1])
        .map_err(|_| "Invalid payload encoding".to_string())?;
    let claims: LicenseClaims = serde_json::from_slice(&payload_bytes)
        .map_err(|_| "Invalid license token claims".to_string())?;

    Ok(claims)
}

/// Generate a best-effort device fingerprint (no private key involved).
fn device_fingerprint() -> String {
    let mut parts = vec![
        whoami_hostname(),
        std::env::consts::OS.to_string(),
        std::env::consts::ARCH.to_string(),
    ];
    if let Ok(user) = std::env::var("USERNAME") {
        parts.push(user);
    }
    if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
        parts.push(appdata);
    }
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(parts.join("|").as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Minimal hostname lookup without extra dependency.
fn whoami_hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown-host".to_string())
}

#[command]
pub async fn activate_device(
    merchant_id: String,
    device_name: String,
) -> Result<LicenseStateResult, String> {
    let client = reqwest::Client::new();
    let base = resolve_api_base(None).await;

    // 1. Request an activation challenge from the control plane.
    let challenge: ActivationChallengeResponse = client
        .post(format!("{}/api/v1/devices/activation-challenge", base))
        .json(&serde_json::json!({
            "merchant_id": merchant_id,
            "device_fingerprint": device_fingerprint(),
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to contact control plane: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Invalid challenge response: {}", e))?;

    // 2. Activate the device (signing happens server-side; no private key on desktop).
    let install_id = uuid::Uuid::new_v4().to_string();
    let activate: ActivateResponse = client
        .post(format!("{}/api/v1/devices/activate", base))
        .json(&serde_json::json!({
            "merchant_id": merchant_id,
            "device_name": device_name,
            "install_id": install_id,
            "device_fingerprint": device_fingerprint(),
            "device_public_key": "",
            "challenge_id": challenge.challenge_id,
            "challenge_response": challenge.challenge,
            "app_version": env!("CARGO_PKG_VERSION"),
            "os": std::env::consts::OS,
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to activate device: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Invalid activation response: {}", e))?;

    // 3. Verify the returned token locally before storing it.
    let claims = verify_jwt(&activate.license_token)?;

    let path = get_license_path();
    let stored = serde_json::json!({
        "token": activate.license_token,
        "server_time": activate.server_time,
    });
    let json_str = serde_json::to_string_pretty(&stored).map_err(|e| e.to_string())?;
    fs::write(&path, json_str).map_err(|e| e.to_string())?;

    // Record server time as the baseline last-seen to detect clock rollback.
    let last_seen_path = get_last_seen_path();
    let _ = fs::write(&last_seen_path, activate.server_time);

    let token = LicenseToken::from_jwt(&activate.license_token, &claims);
    Ok(LicenseStateResult {
        mode: if token.status == "active" { "Active".into() } else { token.status.clone() },
        token: Some(token),
        error: None,
    })
}

#[command]
pub async fn verify_license() -> Result<LicenseStateResult, String> {
    let path = get_license_path();

    if !path.exists() {
        return Ok(LicenseStateResult {
            mode: "Unlicensed".into(),
            token: None,
            error: Some("No license found. Please activate device.".into()),
        });
    }

    let json_str = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return Ok(LicenseStateResult {
            mode: "Unlicensed".into(),
            token: None,
            error: Some("Failed to read license file.".into()),
        }),
    };

    let stored: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return Ok(LicenseStateResult {
            mode: "Unlicensed".into(),
            token: None,
            error: Some("Invalid license format.".into()),
        }),
    };

    let raw = stored
        .get("token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing token in license file".to_string())?;

    // Verify Ed25519 signature and decode claims (server public key only).
    let claims = match verify_jwt(raw) {
        Ok(c) => c,
        Err(_) => return Ok(LicenseStateResult {
            mode: "Unlicensed".into(),
            token: None,
            error: Some("License token signature is invalid. TAMPERED!".into()),
        }),
    };

    let now = Utc::now();

    // SEC-002: Clock Rollback Bypass Vulnerability Fix
    let last_seen_path = get_last_seen_path();
    let last_seen = if last_seen_path.exists() {
        let content = fs::read_to_string(&last_seen_path).unwrap_or_default();
        DateTime::parse_from_rfc3339(content.trim())
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap().with_timezone(&Utc))
    } else {
        DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap().with_timezone(&Utc)
    };

    if now < last_seen {
        let token = LicenseToken::from_jwt(raw, &claims);
        return Ok(LicenseStateResult {
            mode: "SuspiciousTime".into(),
            token: Some(token),
            error: Some("Clock rollback detected. Please correct your system time.".into()),
        });
    }

    // Persist current server/network time baseline (prefer stored server_time if newer).
    let stored_server_time = stored
        .get("server_time")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or(last_seen);
    let baseline = stored_server_time.max(now);
    let _ = fs::write(&last_seen_path, baseline.to_rfc3339());

    let mut mode = "Active".to_string();
    if now.timestamp() > claims.grace_until {
        mode = "RestrictedExpired".to_string();
    } else if now.timestamp() > claims.exp {
        mode = "Grace".to_string();
    }

    let token = LicenseToken::from_jwt(raw, &claims);
    Ok(LicenseStateResult {
        mode,
        token: Some(token),
        error: None,
    })
}

// SEC-001: Centralized Active License Enforcer
pub async fn enforce_active_license() -> Result<(), String> {
    let state = verify_license().await?;
    if state.mode == "RestrictedExpired" || state.mode == "Unlicensed" || state.mode == "SuspiciousTime" {
        return Err(format!(
            "AKSES DITOLAK: Mode lisensi saat ini ({}) memblokir operasi ini. Silakan periksa koneksi internet atau perbarui langganan Anda.",
            state.mode
        ));
    }
    Ok(())
}

#[command]
pub async fn refresh_license() -> Result<LicenseStateResult, String> {
    // Read the existing device id from the stored token to refresh.
    let path = get_license_path();
    if !path.exists() {
        return Err("Cannot refresh: no license found. Please activate first.".into());
    }
    let json_str = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let stored: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| e.to_string())?;
    let raw = stored
        .get("token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing token in license file".to_string())?;
    let claims = verify_jwt(raw)?;

    let client = reqwest::Client::new();
    let base = resolve_api_base(None).await;

    // Refresh with the control plane (server re-signs; no private key on desktop).
    let refresh: RefreshResponse = client
        .post(format!("{}/api/v1/licenses/refresh", base))
        .json(&serde_json::json!({
            "device_id": claims.device_id,
            "nonce": uuid::Uuid::new_v4().to_string(),
            "signature": "",
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to refresh license: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Invalid refresh response: {}", e))?;

    let new_claims = verify_jwt(&refresh.license_token)?;

    let stored = serde_json::json!({
        "token": refresh.license_token,
        "server_time": refresh.server_time,
    });
    let json_str = serde_json::to_string_pretty(&stored).map_err(|e| e.to_string())?;
    fs::write(&path, json_str).map_err(|e| e.to_string())?;

    let last_seen_path = get_last_seen_path();
    let _ = fs::write(&last_seen_path, refresh.server_time);

    let token = LicenseToken::from_jwt(&refresh.license_token, &new_claims);
    Ok(LicenseStateResult {
        mode: if token.status == "active" { "Active".into() } else { token.status.clone() },
        token: Some(token),
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;
    use rand_core::RngCore;

    // Build a JWT signed by an arbitrary Ed25519 key, for negative testing.
    fn forge_jwt(claims: &LicenseClaims, signer: &ed25519_dalek::SigningKey) -> String {
        let header_b64 = BASE64.encode(br#"{"alg":"EdDSA","typ":"JWT"}"#);
        let payload = serde_json::to_vec(claims).unwrap();
        let payload_b64 = BASE64.encode(&payload);
        let signing_input = format!("{}.{}", header_b64, payload_b64);
        let sig = signer.sign(signing_input.as_bytes());
        let sig_b64 = BASE64.encode(sig.to_bytes());
        format!("{}.{}.{}", header_b64, payload_b64, sig_b64)
    }

    fn random_signing_key() -> ed25519_dalek::SigningKey {
        let mut b = [0u8; 32];
        rand_core::OsRng.fill_bytes(&mut b);
        ed25519_dalek::SigningKey::from_bytes(&b)
    }

    fn sample_claims() -> LicenseClaims {
        let now = Utc::now().timestamp();
        LicenseClaims {
            merchant_id: "m1".into(),
            device_id: "d1".into(),
            runtime_mode: "active".into(),
            iat: now,
            nbf: now,
            exp: now + 86400 * 30,
            grace_until: now + 86400 * 37,
        }
    }

    #[test]
    fn verify_jwt_rejects_unknown_signer() {
        // A token signed by a random key (not the server public key) must be rejected.
        let rogue = random_signing_key();
        let token = forge_jwt(&sample_claims(), &rogue);
        assert!(verify_jwt(&token).is_err(), "forged token must fail verification");
    }

    #[test]
    fn verify_jwt_rejects_malformed_token() {
        assert!(verify_jwt("not.a.jwt").is_err());
        assert!(verify_jwt("garbage").is_err());
    }

    #[test]
    fn verify_jwt_rejects_tampered_payload() {
        // Sign a valid token, then flip a claim after signing (tamper).
        let key = random_signing_key();
        let mut claims = sample_claims();
        let token = forge_jwt(&claims, &key);
        // Re-sign with a different exp so the signature no longer matches the payload.
        claims.exp = claims.exp + 1;
        let tampered = forge_jwt(&claims, &key);
        // The tampered token's signature is over the new payload; but the embedded payload
        // in `tampered` is the new one, so to truly tamper we substitute payloads:
        let parts: Vec<&str> = token.split('.').collect();
        let tampered2 = format!("{}.{}.{}", parts[0], parts[1], tampered.split('.').nth(2).unwrap());
        assert!(verify_jwt(&tampered2).is_err(), "tampered token must fail");
    }
}

