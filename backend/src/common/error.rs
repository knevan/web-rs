use axum::{Json, http::StatusCode};
use axum_core::response::{IntoResponse, Response};

// Custom error type definition
pub enum AuthError {
    InvalidToken,
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidRefreshToken,
}

// Implement IntoResponse so that error can be converted into an HTTP response
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create token")
            }
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::InvalidRefreshToken => (StatusCode::UNAUTHORIZED, "Invalid refresh token"),
        };

        let body = Json(serde_json::json!({"error": error_message}));

        (status, body).into_response()
    }
}
