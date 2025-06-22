use crate::common::error::AuthError;
use crate::common::jwt::{Claims, RefreshClaims, create_access_jwt, create_refresh_jwt};
use axum::Json;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
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
/// Accepts a `CookieJar` and return modified `CookieJar`
/// with the token set as a cookie
pub async fn login_handler(
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<JsonValue>), AuthError> {
    // Simple validation
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err(AuthError::MissingCredentials);
    }

    if payload.username != "admin" || payload.password != "admin123" {
        return Err(AuthError::WrongCredentials);
    }

    // Generate access token and refresh token
    let access_token = create_access_jwt(payload.username.clone())?;
    let refresh_token = create_refresh_jwt(payload.username)?;

    // Set cookie for access tokens
    let access_cookie = Cookie::build(("token", access_token))
        .path("/")
        .http_only(true)
        .secure(false) // Only send via HTTPS (disable for local development)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::minutes(15 * 60))
        .build();

    // Set cookie for refresh tokens
    let refresh_cookie = Cookie::build(("refresh_token", refresh_token))
        .path("/")
        .http_only(true)
        .secure(false) // Only send via HTTPS (disable for local development)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::days(7))
        .build();

    // Add both cookie to the jar
    let new_jar = jar.add(access_cookie).add(refresh_cookie);

    // Return success response
    let response = serde_json::json!({
       "message": "Login Success",
    });

    Ok((new_jar, Json(response)))
}

/// Handler for refresh access token
pub async fn refresh_token_handler(
    jar: CookieJar,
    claims: RefreshClaims,
) -> Result<(CookieJar, Json<JsonValue>), AuthError> {
    let new_access_token = create_access_jwt(claims.sub)?;

    let new_access_cookie = Cookie::build(("token", new_access_token))
        .path("/")
        .http_only(true)
        .secure(false) // Only send via HTTPS (disable for local development)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::minutes(15))
        .build();

    let new_jar = jar.add(new_access_cookie);
    let response_body = serde_json::json!({
        "message": "Token refreshed successfully",
    });

    Ok((new_jar, Json(response_body)))
}

/// Handler for logout
pub async fn logout_handler(jar: CookieJar) -> Result<(CookieJar, Json<JsonValue>), AuthError> {
    // Clear all cookies
    let access_removal_cookie = Cookie::build("token")
        .path("/")
        .max_age(time::Duration::ZERO)
        .build();

    // Clear refresh token cookie
    let refresh_removal_cookie = Cookie::build("refresh_token")
        .path("/")
        .max_age(time::Duration::ZERO)
        .build();

    let new_jar = jar.add(access_removal_cookie).add(refresh_removal_cookie);
    let response_body = serde_json::json!({
        "message": "Logged Successful"
    });

    Ok((new_jar, Json(response_body)))
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
