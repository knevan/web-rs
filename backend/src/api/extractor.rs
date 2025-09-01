use crate::builder::startup::AppState;
use crate::common::error::AuthError;
use crate::common::jwt::Claims;
use crate::database::Users;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_core::__private::tracing::error;

pub struct AuthenticatedUser {
    pub id: i32,
    pub username: String,
    pub role_id: i32,
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let claims = Claims::from_request_parts(parts, state)
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let user = state
            .db_service
            .get_user_by_identifier(&claims.sub)
            .await
            .map_err(|e| {
                error!("Failed to get user identifier from token: {:?}", e);
                AuthError::InternalServerError
            })?
            .ok_or(AuthError::InvalidToken)?;

        Ok(AuthenticatedUser {
            id: user.id,
            username: user.username,
            role_id: user.role_id,
        })
    }
}

pub struct AdminUser(pub Users);

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let claims = Claims::from_request_parts(parts, state)
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let user = state
            .db_service
            .get_user_by_identifier(&claims.sub)
            .await
            .map_err(|_| AuthError::InternalServerError)?
            .ok_or(AuthError::InvalidToken)?;

        let role_name = state
            .db_service
            .get_role_name_by_id(user.role_id)
            .await
            .map_err(|_| AuthError::InternalServerError)?
            .ok_or(AuthError::InvalidToken)?;

        if role_name != "admin" {
            return Err(AuthError::WrongCredentials);
        }

        Ok(AdminUser(user))
    }
}
