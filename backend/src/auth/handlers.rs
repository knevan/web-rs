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
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

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

/// Handler for login request
/// Accepts a `CookieJar` and return modified `CookieJar`
/// with the token set as a cookie
pub async fn login_handler(
    jar: CookieJar,
    State(db_service): State<DatabaseService>,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), AuthError> {
    // Simple validation
    if payload.identifier.is_empty() || payload.password.is_empty() {
        return Err(AuthError::MissingCredentials);
    }

    // Find user in the database by either username or email
    let user = db_service
        .get_user_by_identifier(&payload.identifier)
        .await
        .map_err(|e| {
            eprintln!("Database error on user lookup: {}", e);
            AuthError::WrongCredentials
        })?
        .ok_or(AuthError::WrongCredentials)?;

    // Verify the password
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

    // Generate access token and refresh token
    let access_token =
        create_access_jwt(user.username.clone(), user.role.clone())?;
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

    // Return success response
    let response = LoginResponse {
        message: "Login Successfull".to_string(),
        user: UserData {
            username: user.username,
            role: user.role,
        },
    };

    Ok((new_jar, Json(response)))
}

/// Handler for refresh access token
pub async fn refresh_token_handler(
    jar: CookieJar,
    claims: RefreshClaims,
) -> Result<(CookieJar, Json<GenericMessageResponse>), AuthError> {
    // This part would need a database lookup in a real app to get the user's role
    // For this example, we'll assume the role based on the username
    // In a real app, you would look up the user's role from the database
    // to ensure their permissions haven't changed.
    let role = if claims.sub == "admin-dashboard" {
        "admin-dashboard".to_string()
    } else {
        "user".to_string()
    };

    let new_access_token = create_access_jwt(claims.sub.clone(), role.clone())?;

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

/// Handler for logout
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
/// If the token is invalid, `from_request_parts` will return `AuthError`,
/// and Axum will automatically convert it to a response error.
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
    State(db_service): State<DatabaseService>,
    Json(payload): Json<RegisterPayload>,
) -> Result<(StatusCode, Json<GenericMessageResponse>), AuthError> {
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

    // Create a new user in the database
    let _new_user = db_service
        .create_user(
            &payload.username,
            &payload.email,
            &hashed_password,
            "user",
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
    State(db_service): State<DatabaseService>,
    Json(payload): Json<CheckUsernamePayload>,
) -> (StatusCode, Json<CheckUsernameResponse>) {
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
