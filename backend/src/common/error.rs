use axum::{Json, http::StatusCode};
use axum_core::response::{IntoResponse, Response};

// Custom error type definition
pub enum AuthError {
    InvalidToken,
    WrongCredentials,
    MissingCredentials,
    InvalidCredentials,
    TokenCreation,
    InvalidRefreshToken,
    InvalidCharacter(String),
    UserAlreadyExists { field: String },
    InternalServerError,
}

// Implement IntoResponse so that error can be converted into an HTTP response
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid credentials")
            }
            AuthError::MissingCredentials => {
                (StatusCode::BAD_REQUEST, "Missing credentials")
            }
            AuthError::InvalidCredentials => {
                (StatusCode::BAD_REQUEST, "Missing credentials")
            }
            AuthError::TokenCreation => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create token")
            }
            AuthError::InvalidToken => {
                (StatusCode::UNAUTHORIZED, "Invalid token")
            }
            AuthError::InvalidRefreshToken => {
                (StatusCode::UNAUTHORIZED, "Invalid refresh token")
            }
            AuthError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AuthError::UserAlreadyExists { field } => {
                return (
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({"message": format!("{} Already exists", field) })),
                    )
                    .into_response();
            }
            AuthError::InvalidCharacter(message) => {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(serde_json::json!({"message": message})),
                )
                    .into_response();
            }
        };

        let body = Json(serde_json::json!({"message": error_message}));
        (status, body).into_response()
    }
}

/// This implementation allow `?` for error on other crates
impl From<anyhow::Error> for AuthError {
    fn from(_: anyhow::Error) -> Self {
        AuthError::InternalServerError
    }
}
