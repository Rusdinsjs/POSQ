#![allow(dead_code)]

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;

use crate::error::ApiError;
use crate::services::auth::validate_token;

#[derive(Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub merchant_id: Option<String>,
}

pub async fn auth_middleware(
    State(_pool): State<PgPool>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError::Unauthorized("Missing Authorization header".to_string()))?;

    // Check for Bearer token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(ApiError::Unauthorized("Invalid Authorization format".to_string()))?;

    // Validate token
    let claims = validate_token(token)?;

    // Create auth context
    let auth_context = AuthContext {
        user_id: claims.sub,
        email: claims.email,
        role: claims.role,
        merchant_id: claims.merchant_id,
    };

    // Insert auth context into request extensions
    request.extensions_mut().insert(auth_context);

    // Continue to the handler
    Ok(next.run(request).await)
}

pub async fn admin_auth_middleware(
    State(_pool): State<PgPool>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError::Unauthorized("Missing Authorization header".to_string()))?;

    // Check for Bearer token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(ApiError::Unauthorized("Invalid Authorization format".to_string()))?;

    // Validate token
    let claims = validate_token(token)?;

    // Check if user is admin
    if !claims.role.starts_with("admin") {
        return Err(ApiError::Forbidden("Admin access required".to_string()));
    }

    // Create auth context
    let auth_context = AuthContext {
        user_id: claims.sub,
        email: claims.email,
        role: claims.role,
        merchant_id: claims.merchant_id,
    };

    // Insert auth context into request extensions
    request.extensions_mut().insert(auth_context);

    // Continue to the handler
    Ok(next.run(request).await)
}

pub struct ExtractedClaims(pub AuthContext);

impl ExtractedClaims {
    pub fn from_parts(parts: &axum::http::request::Parts) -> Result<Self, ApiError> {
        let auth_context = parts
            .extensions
            .get::<AuthContext>()
            .ok_or(ApiError::Unauthorized("Not authenticated".to_string()))?;

        Ok(ExtractedClaims(auth_context.clone()))
    }
}
