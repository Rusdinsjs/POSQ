use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub merchant_id: Option<String>,
    pub token_type: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| ApiError::InternalError(format!("Failed to hash password: {}", e)))?
        .to_string();
    Ok(password_hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, ApiError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| ApiError::InternalError(format!("Failed to parse password hash: {}", e)))?;
    
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

fn get_jwt_secret() -> Result<String, ApiError> {
    std::env::var("JWT_SECRET")
        .map_err(|_| ApiError::InternalError("JWT_SECRET not configured".to_string()))
}

pub fn generate_tokens(
    user_id: &str,
    email: &str,
    role: &str,
    merchant_id: Option<&str>,
) -> Result<TokenPair, ApiError> {
    let secret = get_jwt_secret()?;
    let now = chrono::Utc::now().timestamp() as usize;
    
    let jwt_expiration_hours: u64 = std::env::var("JWT_EXPIRATION_HOURS")
        .unwrap_or_else(|_| "24".to_string())
        .parse()
        .unwrap_or(24);
    
    let jwt_refresh_expiration_days: u64 = std::env::var("JWT_REFRESH_EXPIRATION_DAYS")
        .unwrap_or_else(|_| "7".to_string())
        .parse()
        .unwrap_or(7);

    // Access token
    let access_claims = TokenClaims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        merchant_id: merchant_id.map(|s| s.to_string()),
        token_type: "access".to_string(),
        exp: now + (jwt_expiration_hours * 3600) as usize,
        iat: now,
    };

    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| ApiError::InternalError(format!("Failed to generate access token: {}", e)))?;

    // Refresh token
    let refresh_claims = TokenClaims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        merchant_id: merchant_id.map(|s| s.to_string()),
        token_type: "refresh".to_string(),
        exp: now + (jwt_refresh_expiration_days * 86400) as usize,
        iat: now,
    };

    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| ApiError::InternalError(format!("Failed to generate refresh token: {}", e)))?;

    Ok(TokenPair {
        access_token,
        refresh_token,
    })
}

pub fn validate_token(token: &str) -> Result<TokenClaims, ApiError> {
    let secret = get_jwt_secret()?;
    let token_data = decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| ApiError::Unauthorized(format!("Invalid token: {}", e)))?;

    Ok(token_data.claims)
}

pub fn validate_refresh_token(token: &str) -> Result<TokenClaims, ApiError> {
    let claims = validate_token(token)?;
    
    if claims.token_type != "refresh" {
        return Err(ApiError::Unauthorized("Not a refresh token".to_string()));
    }

    Ok(claims)
}
