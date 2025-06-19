use crate::common::error::AuthError;
use crate::common::jwt::{Claims, create_jwt};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    message: String,
    token: String,
}

/// Handler for login request
pub async fn login_handler(
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AuthError> {
    // Simple validation
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err(AuthError::MissingCredentials);
    }

    if payload.username != "admin" || payload.password != "admin123" {
        return Err(AuthError::WrongCredentials);
    }

    // Generate a JWT token
    let token = create_jwt(payload.username)?;

    // Return success response
    let response = LoginResponse {
        message: "Login Success".to_string(),
        token,
    };

    Ok(Json(response))
}

/// Protected handler. `Claims` acts as a guard.
/// If the token is invalid, `from_request_parts` will return `AuthError`,
/// and Axum will automatically convert it to a response error.
pub async fn protected_handler(claims: Claims) -> Json<JsonValue> {
    // This handler will only be called if `claims` is extracted successfully (valid token)
    let response = serde_json::json!({
        "message": "Welcome to the protected area",
        "user_id": claims.sub,
        "session_expired": claims.exp,
    });
    Json(response)
}
