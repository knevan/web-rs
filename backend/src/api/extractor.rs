use std::convert::Infallible;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_core::__private::tracing::error;

use crate::builder::startup::AppState;
use crate::common::error::AuthError;
use crate::common::jwt::Claims;
use crate::database::Users;

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum Role {
    User = 0,
    Moderator = 1,
    Admin = 2,
    SuperAdmin = 3,
}

impl Role {
    pub fn from_name(role_name: &str) -> Option<Self> {
        match role_name {
            "superadmin" => Some(Role::SuperAdmin),
            "admin" => Some(Role::Admin),
            "moderator" => Some(Role::Moderator),
            "user" => Some(Role::User),
            _ => None,
        }
    }

    pub fn to_name(&self) -> &str {
        match self {
            Role::SuperAdmin => "superadmin",
            Role::Admin => "admin",
            Role::Moderator => "moderator",
            Role::User => "user",
        }
    }
}

// Authenticated user
pub struct AuthenticatedUser {
    pub id: i32,
    pub username: String,
    pub role: Role,
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let claims = Claims::from_request_parts(parts, state)
            .await
            .map_err(|_err| AuthError::InvalidToken)?;

        let user = state
            .db_service
            .get_user_by_identifier(&claims.sub)
            .await
            .map_err(|e| {
                error!("Failed to get user identifier from token: {:?}", e);
                AuthError::InternalServerError
            })?
            .ok_or(AuthError::InvalidToken)?;

        let role_name = state
            .db_service
            .get_role_name_by_id(user.role_id)
            .await
            .map_err(|_err| AuthError::InternalServerError)?
            .ok_or(AuthError::InvalidToken)?;

        let role = Role::from_name(&role_name).ok_or_else(|| {
            error!("Role name '{}' in DB is not a valid Role enum.", role_name);
            AuthError::InternalServerError
        })?;

        Ok(AuthenticatedUser {
            id: user.id,
            username: user.username,
            role,
        })
    }
}

// Optional authenticated user (sign-in or sign-out)
pub struct OptionalAuthenticatedUser(pub Option<AuthenticatedUser>);

impl FromRequestParts<AppState> for OptionalAuthenticatedUser {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user_result = AuthenticatedUser::from_request_parts(parts, state).await;

        // If extraction is successful, wrap it in Some.
        // we treat it as None instead of rejecting the request.
        let user = user_result.ok();

        Ok(OptionalAuthenticatedUser(user))
    }
}

// Super admin user only
pub struct SuperAdminUser(pub AuthenticatedUser);

impl FromRequestParts<AppState> for SuperAdminUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state).await?;

        if user.role != Role::SuperAdmin {
            Err(AuthError::WrongCredentials)
        } else {
            Ok(SuperAdminUser(user))
        }
    }
}

// Admin or higher user
pub struct AdminOrHigherUser(pub AuthenticatedUser);

impl FromRequestParts<AppState> for AdminOrHigherUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state).await?;

        if user.role < Role::Admin {
            Err(AuthError::WrongCredentials)
        } else {
            Ok(AdminOrHigherUser(user))
        }
    }
}

// Moderator or higher user
pub struct ModeratorOrHigherUser(pub AuthenticatedUser);

impl FromRequestParts<AppState> for ModeratorOrHigherUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state).await?;

        if user.role < Role::Moderator {
            Err(AuthError::WrongCredentials)
        } else {
            Ok(ModeratorOrHigherUser(user))
        }
    }
}
