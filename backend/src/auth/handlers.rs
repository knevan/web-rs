use crate::builder::startup::AppState;
use crate::common::email_service::send_password_reset_email;
use crate::common::error::AuthError;
use crate::common::hashing::{hash_password, verify_password};
use crate::common::jwt::{
    Claims, RefreshClaims, create_access_jwt, create_refresh_jwt,
};
use crate::db::db::DatabaseService;
use axum::Json;
use axum::extract::State;
use axum_core::__private::tracing::error;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{Duration, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Struct for payloads
#[derive(Deserialize)]
pub struct LoginRequest {
    identifier: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RegisterPayload {
    username: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct CheckUsernamePayload {
    username: String,
}

#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

// Struct for Responses
#[derive(Serialize)]
pub struct UserData {
    username: String,
    role: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    message: String,
    user: UserData,
}

#[derive(Serialize)]
pub struct ProtectedResponse {
    message: String,
    user_id: String,
    session_expires_at: i64,
}

#[derive(Serialize)]
pub struct GenericMessageResponse {
    message: String,
}

#[derive(Serialize)]
pub struct CheckUsernameResponse {
    available: bool,
    message: String,
}

// Helper function to get role name string from role id
async fn get_role_name(
    db_service: &DatabaseService,
    role_id: i32,
) -> Result<String, AuthError> {
    db_service
        .get_role_name_by_id(role_id)
        .await
        .map_err(|e| {
            error!("Failed to get role name by id: {}", e);
            AuthError::InternalServerError
        })?
        .ok_or_else(|| {
            error!("Role with id {} not found.", role_id);
            AuthError::InternalServerError
        })
}

// Accepts a `CookieJar` and return modified `CookieJar`
// with the token set as a cookie
pub async fn login_handler(
    jar: CookieJar,
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), AuthError> {
    let db_service = &state.db_service;

    // Simple validation
    if payload.identifier.is_empty() || payload.password.is_empty() {
        return Err(AuthError::MissingCredentials);
    }

    // Find user identifier
    let user = db_service
        .get_user_by_identifier(&payload.identifier)
        .await
        .map_err(|e| {
            eprintln!("Database error on user lookup: {}", e);
            AuthError::WrongCredentials
        })?
        .ok_or(AuthError::WrongCredentials)?;

    // Verify
    let is_valid_password =
        verify_password(&payload.password, &user.password_hash).map_err(
            |e| {
                eprintln!("Password verification error: {}", e);
                AuthError::WrongCredentials
            },
        )?;

    if !is_valid_password {
        return Err(AuthError::WrongCredentials);
    }

    let role_name = get_role_name(db_service, user.role_id).await?;

    // Generate access token and refresh token
    let access_token =
        create_access_jwt(user.username.clone(), role_name.clone())?;
    let refresh_token = create_refresh_jwt(user.username.clone())?;

    // Set cookie for access tokens
    let access_cookie = Cookie::build(("token", access_token))
        .path("/")
        .http_only(true)
        .secure(false) // Only send via HTTPS (disable for local development)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(15 * 60))
        .build();

    // Set cookie for refresh tokens
    let refresh_cookie = Cookie::build(("refresh-token", refresh_token))
        .path("/")
        .http_only(true)
        .secure(false) // Only send via HTTPS (disable for local development)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::days(7))
        .build();

    // Add both cookie to the jar
    let new_jar = jar.add(access_cookie).add(refresh_cookie);

    let response = LoginResponse {
        message: "Login Successfull".to_string(),
        user: UserData {
            username: user.username,
            role: role_name,
        },
    };

    Ok((new_jar, Json(response)))
}

pub async fn refresh_token_handler(
    jar: CookieJar,
    State(state): State<AppState>,
    claims: RefreshClaims,
) -> Result<(CookieJar, Json<GenericMessageResponse>), AuthError> {
    let user = state
        .db_service
        .get_user_by_identifier(&claims.sub)
        .await
        .map_err(|_| AuthError::InvalidToken)?
        .ok_or(AuthError::InvalidToken)?;

    let role_name = get_role_name(&state.db_service, user.role_id).await?;

    let new_access_token =
        create_access_jwt(claims.sub.clone(), role_name.clone())?;

    /*let new_access_token = create_access_jwt(claims.sub)?;*/

    let new_access_cookie = Cookie::build(("token", new_access_token))
        .path("/")
        .http_only(true)
        .secure(false) // Only send via HTTPS (disable for local development)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::minutes(15))
        .build();

    let new_jar = jar.add(new_access_cookie);
    let response_body = GenericMessageResponse {
        message: "Token refreshed successfully".to_string(),
    };

    Ok((new_jar, Json(response_body)))
}

pub async fn logout_handler(
    jar: CookieJar,
) -> Result<(CookieJar, Json<GenericMessageResponse>), AuthError> {
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
    let response_body = GenericMessageResponse {
        message: "Logged out successfully".to_string(),
    };

    Ok((new_jar, Json(response_body)))
}

/// Protected handler. `Claims` acts as a guard.
pub async fn protected_handler(claims: Claims) -> Json<ProtectedResponse> {
    println!(
        "[API] Request received at /api/auth/user for user: {}",
        claims.sub
    );

    // This handler will only be called if `claims` is extracted successfully (valid token)
    let response = ProtectedResponse {
        message: "Welcome to protected area".to_string(),
        user_id: claims.sub,
        session_expires_at: claims.exp as i64,
    };
    Json(response)
}

/// Handler for registering new users
/// it checks for uniqueness and create new user in the database
pub async fn register_user_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<(StatusCode, Json<GenericMessageResponse>), AuthError> {
    let db_service = &state.db_service;

    // Validate input: ensure no fields are empty
    if payload.username.is_empty()
        || payload.email.is_empty()
        || payload.password.is_empty()
    {
        return Err(AuthError::MissingCredentials);
    }

    // Check if user already exists in the database
    // We use existing `get_user_by_identifier` method
    if db_service
        .get_user_by_identifier(&payload.username)
        .await?
        .is_some()
    {
        return Err(AuthError::UserAlreadyExists {
            field: "username".to_string(),
        });
    }
    if db_service
        .get_user_by_identifier(&payload.email)
        .await?
        .is_some()
    {
        return Err(AuthError::UserAlreadyExists {
            field: "email".to_string(),
        });
    }

    // Hash the password before storing it in the database
    let hashed_password = hash_password(&payload.password)
        .map_err(|_| AuthError::InternalServerError)?;

    // Get the ID for the default 'user' role.
    let user_role_id = db_service
        .get_role_id_by_name("user")
        .await
        .map_err(|_| AuthError::InternalServerError)?
        .ok_or_else(|| {
            error!("Default 'user' role not found in the database.");
            AuthError::InternalServerError
        })?;

    // Create a new user in the database
    let _new_user = db_service
        .create_user(
            &payload.username,
            &payload.email,
            &hashed_password,
            user_role_id,
        )
        .await
        .map_err(|_| AuthError::InternalServerError)?;

    // Return success response
    let response = GenericMessageResponse {
        message: "User registered successfully".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

// Handler for real-time checking username availability
pub async fn check_username_handler(
    State(state): State<AppState>,
    Json(payload): Json<CheckUsernamePayload>,
) -> (StatusCode, Json<CheckUsernameResponse>) {
    let db_service = &state.db_service;

    // Validate username length on the backend as well
    if payload.username.trim().len() < 4 {
        let response = CheckUsernameResponse {
            available: false,
            message: "Username should be at least 4 characters long"
                .to_string(),
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }

    match db_service.get_user_by_identifier(&payload.username).await {
        Ok(Some(_)) => {
            // Username not available
            let response = CheckUsernameResponse {
                available: false,
                message: "Username already exists".to_string(),
            };
            (StatusCode::OK, Json(response))
        }
        Ok(None) => {
            // No user found, username available
            let response = CheckUsernameResponse {
                available: true,
                message: "Username is available".to_string(),
            };
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            error!(
                "Database error when checking username availability: {:?}",
                e
            );
            let response = CheckUsernameResponse {
                available: false,
                message:
                    "Error checking username availability. Please try again"
                        .to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

// Handler for the password reset request
// Finds a user by email, generates a token, and in a real app, sends an email.
pub async fn forgot_password_handler(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<Json<GenericMessageResponse>, AuthError> {
    let db_service = &state.db_service;

    // Find user by email
    if let Ok(Some(user)) =
        db_service.get_user_by_identifier(&payload.email).await
    {
        // Generate unique password reset token
        let token = Uuid::new_v4().to_string();
        // Token valid for 1 hour
        let expired_at = Utc::now() + Duration::hours(1);

        // Store reset token in the database
        db_service
            .create_password_reset_token(user.id, &token, expired_at)
            .await
            .map_err(|_| AuthError::InternalServerError)?;

        // Send the password reset email
        if let Err(e) = send_password_reset_email(
            &state.mailer,
            &user.email,
            &user.username,
            &token,
        )
        .await
        {
            // Error log if sending email fails
            eprintln!(
                "[AUTH HANDLER] Failed to send password reset email: {:?}",
                e
            );
        }
    }

    let response = GenericMessageResponse {
        message: "Password reset request sent. Check your email for further instructions"
            .to_string(),
    };

    Ok(Json(response))
}

// Handler for finalizing password reset with token
pub async fn reset_password_handler(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<Json<GenericMessageResponse>, AuthError> {
    let db_service = &state.db_service;

    // Validate input: ensure token and password not empty
    if payload.token.is_empty() || payload.new_password.is_empty() {
        return Err(AuthError::MissingCredentials);
    }

    // Find user and token details from database
    let (user_id, expires_at) = db_service
        .get_user_by_reset_token(&payload.token)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    // Check if token is expired
    if Utc::now() > expires_at {
        // Clean up expired tokens
        db_service
            .delete_password_reset_token(&payload.token)
            .await?;
        return Err(AuthError::MissingCredentials);
    }

    // Hash the new password before storing it in the database
    let hashed_password = hash_password(&payload.new_password)
        .map_err(|_| AuthError::InvalidCredentials)?;

    // Update user's password in the database
    db_service
        .update_user_password_hash_after_reset_password(
            user_id,
            &hashed_password,
        )
        .await
        .map_err(|_| AuthError::InternalServerError)?;

    // Invalidate token by deleting it after successful use
    db_service
        .delete_password_reset_token(&payload.token)
        .await?;

    let response = GenericMessageResponse {
        message: "Password reset successful".to_string(),
    };

    Ok(Json(response))
}
