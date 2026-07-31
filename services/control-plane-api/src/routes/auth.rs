use axum::{extract::State, Json};
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::error::ApiError;
use crate::models::{LoginRequest, LoginResponse, RefreshTokenRequest, UserResponse, MerchantResponse};
use crate::services::auth::{verify_password, generate_tokens, validate_refresh_token};

pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // Find user by email
    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM merchant_users WHERE email = $1 AND deleted_at IS NULL"
    )
    .bind(&payload.email)
    .fetch_optional(&pool)
    .await?;

    let user = match user {
        Some(u) => u,
        None => return Err(ApiError::Unauthorized("Invalid email or password".to_string())),
    };

    // Check if account is locked
    if let Some(locked_until) = user.locked_until {
        if locked_until > chrono::Utc::now() {
            return Err(ApiError::Unauthorized("Account is locked. Please try again later".to_string()));
        }
    }

    // Verify password
    if !verify_password(&payload.password, &user.password_hash)? {
        // Increment failed login attempts
        let new_attempts = user.failed_login_attempts + 1;
        let lock_until = if new_attempts >= 5 {
            Some(chrono::Utc::now() + chrono::Duration::minutes(15))
        } else {
            None
        };

        sqlx::query(
            "UPDATE merchant_users SET failed_login_attempts = $1, locked_until = $2 WHERE id = $3"
        )
        .bind(new_attempts)
        .bind(lock_until)
        .bind(user.id)
        .execute(&pool)
        .await?;

        return Err(ApiError::Unauthorized("Invalid email or password".to_string()));
    }

    // Reset failed login attempts on successful login
    sqlx::query(
        "UPDATE merchant_users SET failed_login_attempts = 0, locked_until = NULL, last_login_at = NOW() WHERE id = $1"
    )
    .bind(user.id)
    .execute(&pool)
    .await?;

    // Get merchant info if user has merchant_id
    let merchant = if let Some(merchant_id) = user.merchant_id {
        sqlx::query_as::<_, crate::models::Merchant>(
            "SELECT * FROM merchants WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(merchant_id)
        .fetch_optional(&pool)
        .await?
    } else {
        None
    };

    // Generate tokens
    let tokens = generate_tokens(
        &user.id.to_string(),
        &user.email,
        &user.role,
        user.merchant_id.map(|m| m.to_string()).as_deref(),
    )?;

    // Store refresh token in database (simplified - in production, hash the token)
    // For now, we'll just return the tokens

    Ok(Json(LoginResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            name: user.name,
            role: user.role,
        },
        merchant: merchant.map(|m| MerchantResponse {
            id: m.id.to_string(),
            name: m.name,
            slug: m.slug,
        }),
    }))
}

pub async fn refresh_token(
    State(pool): State<PgPool>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<Value>, ApiError> {
    // Validate refresh token
    let token_data = validate_refresh_token(&payload.refresh_token)?;

    // Check if user exists
    let user_id: uuid::Uuid = token_data.sub.parse().map_err(|_| ApiError::Unauthorized("Invalid token".to_string()))?;

    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM merchant_users WHERE id = $1 AND deleted_at IS NULL"
    )
    .bind(user_id)
    .fetch_optional(&pool)
    .await?;

    let user = match user {
        Some(u) => u,
        None => return Err(ApiError::Unauthorized("User not found".to_string())),
    };

    // Generate new tokens
    let tokens = generate_tokens(
        &user.id.to_string(),
        &user.email,
        &user.role,
        user.merchant_id.map(|m| m.to_string()).as_deref(),
    )?;

    Ok(Json(json!({
        "access_token": tokens.access_token,
        "refresh_token": tokens.refresh_token,
    })))
}

pub async fn logout(
    State(_pool): State<PgPool>,
) -> Result<Json<Value>, ApiError> {
    tracing::info!("User logged out");

    Ok(Json(json!({
        "success": true,
        "message": "Logged out successfully",
        "server_time": chrono::Utc::now().to_rfc3339()
    })))
}
