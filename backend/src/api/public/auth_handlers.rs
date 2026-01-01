use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_core::__private::tracing::error;
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::extractor::AuthenticatedUser;
use crate::builder::startup::AppState;
use crate::common::email_service::send_password_reset_email;
use crate::common::error::AuthError;
use crate::common::hashing::{hash_password, verify_password};
use crate::common::jwt::{create_access_jwt, create_refresh_jwt, RefreshClaims};
use crate::database::DatabaseService;

#[derive(Serialize)]
pub struct GenericMessageResponse {
    message: String,
}

// Helper function to get role name string from role id
async fn get_role_name(db_service: &DatabaseService, role_id: i32) -> Result<String, AuthError> {
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

#[derive(Deserialize)]
pub struct RegisterPayload {
    username: String,
    email: String,
    password: String,
}

impl RegisterPayload {
    fn validate_input(&self) -> Result<(), AuthError> {
        if self.username.trim().len() < 4 {
            return Err(AuthError::InvalidCharacter(
                "Username should be at least 4 characters long.".to_string(),
            ));
        }
        if self.email.is_empty() || self.password.is_empty() {
            return Err(AuthError::MissingCredentials);
        }

        Ok(())
    }
}

/// Register new user
/// Its checks for uniqueness and create a new user in the database
pub async fn register_new_user_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<(StatusCode, Json<GenericMessageResponse>), AuthError> {
    let db_service = &state.db_service;

    // Validate input
    payload.validate_input()?;

    // Check if user/username already exists in the database
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

    let hashed_password =
        hash_password(&payload.password).map_err(|_err| AuthError::InternalServerError)?;

    // Get the ID for the default 'user' role.
    let user_role_id = db_service
        .get_role_id_by_name("user")
        .await
        .map_err(|_err| AuthError::InternalServerError)?
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
        .map_err(|_err| AuthError::InternalServerError)?;

    let response = GenericMessageResponse {
        message: "User registered successfully".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

// Struct for Responses
#[derive(Serialize)]
pub struct UserData {
    id: i32,
    username: String,
    role: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    message: String,
    user: UserData,
}

// Struct for payloads
#[derive(Deserialize)]
pub struct LoginRequest {
    identifier: String,
    password: String,
}

impl LoginRequest {
    fn validate_input(&self) -> Result<(), AuthError> {
        if self.identifier.is_empty() || self.password.is_empty() {
            return Err(AuthError::MissingCredentials);
        }

        Ok(())
    }
}

/// Login authenticated user with credential verification
/// Return access and refresh tokens as HTTP-only cookies
pub async fn login_handler(
    jar: CookieJar,
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), AuthError> {
    payload.validate_input()?;

    let db_service = &state.db_service;

    let user = db_service
        .get_user_by_identifier(&payload.identifier)
        .await
        .map_err(|e| {
            eprintln!("Database error on user lookup: {}", e);
            AuthError::InternalServerError
        })?
        .ok_or(AuthError::WrongCredentials)?;

    let is_password_valid =
        verify_password(&payload.password, &user.password_hash).map_err(|_err| {
            error!("Password verification failed for user {}", user.username);
            AuthError::WrongCredentials
        })?;

    if !is_password_valid {
        return Err(AuthError::WrongCredentials);
    }

    let role_name = get_role_name(db_service, user.role_id).await?;

    let access_token = create_access_jwt(user.username.clone(), role_name.clone())?;
    let refresh_token = create_refresh_jwt(user.username.clone())?;

    // Set cookie
    let access_cookie = Cookie::build(("token", access_token))
        .path("/")
        .http_only(true)
        .secure(false) // Only send via HTTPS (disable for local development)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(30 * 60))
        .build();

    // Set cookie
    let refresh_cookie = Cookie::build(("refresh-token", refresh_token))
        .path("/")
        .http_only(true)
        .secure(false) // Only send via HTTPS (disable for local development)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::days(7))
        .build();

    // Add both to the jar
    let new_jar = jar.add(access_cookie).add(refresh_cookie);

    let response = LoginResponse {
        message: "Login Successfull".to_string(),
        user: UserData {
            id: user.id,
            username: user.username,
            role: role_name,
        },
    };

    Ok((new_jar, Json(response)))
}

/// Logout the user by clearing the token cookies
/// Clear access dan refresh token from the browser
pub async fn logout_handler(
    jar: CookieJar,
) -> Result<(CookieJar, Json<GenericMessageResponse>), AuthError> {
    fn create_expired_cookie(name: &'static str) -> Cookie<'static> {
        Cookie::build(name)
            .path("/")
            .max_age(time::Duration::ZERO)
            .http_only(true)
            .secure(false) // Only send via HTTPS (disable for local development)
            .same_site(SameSite::Lax)
            .build()
    }

    let clear_access_token_cookie = create_expired_cookie("token");
    let clear_refresh_token_cookie = create_expired_cookie("refresh-token");

    let new_jar = jar
        .add(clear_access_token_cookie)
        .add(clear_refresh_token_cookie);

    let response_body = GenericMessageResponse {
        message: "Logged out successfully".to_string(),
    };

    Ok((new_jar, Json(response_body)))
}

/// Refresh the access token using a valid refresh token.
/// Return a new access token in a cookie
pub async fn refresh_access_token_handler(
    jar: CookieJar,
    State(state): State<AppState>,
    claims: RefreshClaims,
) -> Result<(CookieJar, Json<GenericMessageResponse>), AuthError> {
    let user = state
        .db_service
        .get_user_by_identifier(&claims.sub)
        .await
        .map_err(|_err| AuthError::InvalidToken)?
        .ok_or(AuthError::InvalidRefreshToken)?;

    let role_name = get_role_name(&state.db_service, user.role_id).await?;

    let new_access_token = create_access_jwt(claims.sub.clone(), role_name)?;

    let new_access_cookie = Cookie::build(("token", new_access_token))
        .path("/")
        .http_only(true)
        .secure(false) // Only send via HTTPS (disable for local development)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::minutes(30))
        .build();

    let new_jar = jar.add(new_access_cookie);
    let response_body = GenericMessageResponse {
        message: "Token refreshed successfully".to_string(),
    };

    Ok((new_jar, Json(response_body)))
}

#[derive(Serialize)]
pub struct UserResponse {
    user: UserData,
}

/// Protected endpoint that returns authenticated user data
pub async fn protected_handler(
    user: AuthenticatedUser,
) -> Result<(StatusCode, Json<UserResponse>), AuthError> {
    println!(
        "[API] Request received at /api/api/user for user: {}",
        user.username
    );

    let role_name = user.role.to_name();

    let user_data = UserData {
        id: user.id,
        username: user.username,
        role: role_name.to_string(),
    };

    let response = UserResponse { user: user_data };

    Ok((StatusCode::OK, Json(response)))
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

impl ResetPasswordRequest {
    fn validate_input(&self) -> Result<(), AuthError> {
        if self.token.is_empty() || self.new_password.is_empty() {
            return Err(AuthError::MissingCredentials);
        }

        Ok(())
    }
}

/// Handler for nitiate password reset by sending reset token to email
pub async fn forgot_password_handler(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<Json<GenericMessageResponse>, AuthError> {
    // Find user by email
    if let Ok(Some(user)) = state
        .db_service
        .get_user_by_identifier(&payload.email)
        .await
    {
        let unique_reset_token = Uuid::new_v4().to_string();
        let expired_at = Utc::now() + Duration::minutes(10);

        // Store reset token in the database
        state
            .db_service
            .create_password_reset_token(user.id, &unique_reset_token, expired_at)
            .await
            .map_err(|_err| AuthError::InternalServerError)?;

        // Send the password reset email
        if let Err(e) = send_password_reset_email(
            &state.mailer,
            &user.email,
            &user.username,
            &unique_reset_token,
        )
        .await
        {
            // Error log if sending email fails
            error!(
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

/// Handler for finalizing password reset using valid reset token
pub async fn reset_password_handler(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<Json<GenericMessageResponse>, AuthError> {
    payload.validate_input()?;

    let db_service = &state.db_service;

    let (user_id, expires_at) = db_service
        .get_user_by_reset_token(&payload.token)
        .await?
        .ok_or(AuthError::InvalidToken)?;

    // Clean up the expired token
    if Utc::now() > expires_at {
        db_service
            .delete_password_reset_token(&payload.token)
            .await?;
        return Err(AuthError::InvalidToken);
    }

    let hashed_password =
        hash_password(&payload.new_password).map_err(|_err| AuthError::InternalServerError)?;

    db_service
        .update_user_password_hash_after_reset_password(user_id, &hashed_password)
        .await
        .map_err(|_err| AuthError::InternalServerError)?;

    db_service
        .delete_password_reset_token(&payload.token)
        .await?;

    let response = GenericMessageResponse {
        message: "Password reset successful".to_string(),
    };

    Ok(Json(response))
}

#[derive(Deserialize)]
pub struct CheckUsernamePayload {
    username: String,
}

#[derive(Serialize)]
pub struct CheckUsernameResponse {
    available: bool,
    message: String,
}

/// Check if username already taken
/// This aims to guarantee username uniqueness
pub async fn realtime_check_username_handler(
    State(state): State<AppState>,
    Json(payload): Json<CheckUsernamePayload>,
) -> (StatusCode, Json<CheckUsernameResponse>) {
    let db_service = &state.db_service;

    // Validate username length on the backend as well
    if payload.username.trim().len() < 4 {
        let response = CheckUsernameResponse {
            available: false,
            message: "Username should be at least 4 characters long".to_string(),
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
                message: "Error checking username availability. Please try again".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}
