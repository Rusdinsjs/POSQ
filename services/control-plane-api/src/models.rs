#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =============================================================================
// Auth Models
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserResponse,
    pub merchant: Option<MerchantResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPayload {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub merchant_id: Option<String>,
    pub exp: usize,
    pub iat: usize,
}

// =============================================================================
// User Models
// =============================================================================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub merchant_id: Option<Uuid>,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub role: String,
    pub mfa_enabled: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
}

// =============================================================================
// Merchant Models
// =============================================================================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Merchant {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MerchantResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
}

// =============================================================================
// Device Models
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivateDeviceRequest {
    pub merchant_id: String,
    pub device_name: String,
    pub install_id: String,
    pub device_fingerprint: String,
    pub device_public_key: String,
    pub challenge_id: String,
    pub challenge_response: String,
    pub app_version: Option<String>,
    pub os: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivateDeviceResponse {
    pub device_id: String,
    pub license_token: String,
    pub server_time: String,
    pub runtime_mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceChallengeRequest {
    pub merchant_id: String,
    pub device_fingerprint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceChallengeResponse {
    pub challenge_id: String,
    pub challenge: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Device {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub name: String,
    pub install_id_hash: String,
    pub device_fingerprint_hash: Option<String>,
    pub device_public_key_thumbprint: Option<String>,
    pub status: String,
    pub app_version: Option<String>,
    pub os: Option<String>,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// License Models
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshLicenseRequest {
    pub device_id: String,
    pub nonce: String,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshLicenseResponse {
    pub license_token: String,
    pub server_time: String,
    pub runtime_mode: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DeviceLicense {
    pub id: Uuid,
    pub device_id: Uuid,
    pub token_version: i32,
    pub token_hash: String,
    pub signing_key_id: Uuid,
    pub runtime_mode: String,
    pub valid_until: DateTime<Utc>,
    pub grace_until: Option<DateTime<Utc>>,
    pub issued_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct LicenseSigningKey {
    pub id: Uuid,
    pub key_id: String,
    pub algorithm: String,
    pub public_key: String,
    pub encrypted_private_key_ref: Option<String>,
    pub status: String,
    pub rotated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Subscription Models
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ManualRenewalRequest {
    pub merchant_id: String,
    pub subscription_id: String,
    pub days_to_add: i32,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub plan_id: Uuid,
    pub status: String,
    pub valid_until: DateTime<Utc>,
    pub grace_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Plan {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub price_monthly: i32,
    pub price_yearly: Option<i32>,
    pub features: serde_json::Value,
    pub max_devices: i32,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Update Models
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCheckRequest {
    pub os: String,
    pub channel: String,
    pub current_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCheckResponse {
    pub update_available: bool,
    pub version: Option<String>,
    pub sha256: Option<String>,
    pub signature: Option<String>,
    pub download_url: Option<String>,
    pub critical: bool,
    pub release_notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishUpdateRequest {
    pub version: String,
    pub channel: String,
    pub os: String,
    pub min_supported_version: Option<String>,
    pub sha256: String,
    pub download_url: String,
    pub release_notes: Option<String>,
    pub critical: bool,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AppVersion {
    pub id: Uuid,
    pub version: String,
    pub channel: String,
    pub os: String,
    pub min_supported_version: Option<String>,
    pub sha256: String,
    pub signature: String,
    pub signing_key_id: Uuid,
    pub download_url: String,
    pub release_notes: Option<String>,
    pub critical: bool,
    pub published_at: DateTime<Utc>,
}

// =============================================================================
// Backup Models
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupMetadataRequest {
    pub device_id: String,
    pub backup_id: String,
    pub destination_type: String,
    pub logical_storage_ref: Option<String>,
    pub size_bytes: i64,
    pub checksum: String,
    pub encryption_algorithm: String,
    pub encrypted: bool,
    pub app_version: Option<String>,
    pub db_schema_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupMetadata {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub device_id: Uuid,
    pub backup_id: String,
    pub destination_type: String,
    pub logical_storage_ref: Option<String>,
    pub size_bytes: i64,
    pub checksum: String,
    pub encryption_algorithm: String,
    pub encrypted: bool,
    pub app_version: Option<String>,
    pub db_schema_version: Option<String>,
    pub status: String,
    pub failure_code: Option<String>,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Admin Models
// =============================================================================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AdminUser {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub role: String,
    pub mfa_enabled: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevokeDeviceRequest {
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AdminAuditLog {
    pub id: Uuid,
    pub admin_id: Option<Uuid>,
    pub merchant_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Heartbeat Models
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub device_id: String,
    pub nonce: String,
    pub signature: String,
    pub app_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub runtime_mode: String,
    pub server_time: String,
    pub license_token: Option<String>,
}
