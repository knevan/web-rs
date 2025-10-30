use std::env;
use std::sync::LazyLock;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::common::error::AuthError;

// Secret key for JWT signing and encryption
static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret_key = env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");
    Keys::new(secret_key.as_bytes())
});

pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    fn new(secret_key: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret_key),
            decoding: DecodingKey::from_secret(secret_key),
        }
    }
}

/// Claims is a struct that represents the claims in the JWT token.
/// It contains the subject (user ID), expiration time, issued at time, and role.
/// The `Claims` struct is used to encode and decode the JWT tokens.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

/// Custom extractor to get `Claims` from cookie access token.
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract cookie jar from request
        let jar = CookieJar::from_request_parts(parts, state).await.unwrap();

        // Get a Cookie named "token"
        let token_cookie = jar.get("token").ok_or(AuthError::InvalidToken)?;
        let token = token_cookie.value();

        // Decode token with HS512 Algorithm
        let mut validation = Validation::default();
        validation.algorithms = vec![Algorithm::HS512];

        let token_data = decode::<Claims>(token, &KEYS.decoding, &validation)
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

/// Custom extractor to get `RefreshClaims` from cookie refresh token.
impl<S> FromRequestParts<S> for RefreshClaims
where
    S: Send + Sync,
{
    type Rejection = AuthError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state).await.unwrap();

        let refresh_token_cookie = jar.get("refresh-token").ok_or(AuthError::InvalidToken)?;
        let refresh_token = refresh_token_cookie.value();

        // Decode token with HS512 Algorithm
        let mut validation = Validation::default();
        validation.algorithms = vec![Algorithm::HS512];

        let token_data = decode::<RefreshClaims>(refresh_token, &KEYS.decoding, &validation)
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

/// Create jwt token for a given user ID and role (access token)
pub fn create_access_jwt(user_id: String, role: String) -> Result<String, AuthError> {
    let now = Utc::now();
    let iat = now.timestamp() as usize;

    // Access token valid for 30 minutes
    let exp = (now + Duration::minutes(30)).timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        role,
        exp,
        iat,
    };

    // Specify HS512 algorithm in the header
    encode(&Header::new(Algorithm::HS512), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)
}

/// Create refresh jwt token for a given user ID (refresh token)
pub fn create_refresh_jwt(user_id: String) -> Result<String, AuthError> {
    let now = Utc::now();
    let iat = now.timestamp() as usize;

    // Refresh token valid for 7 days
    let exp = (now + Duration::days(7)).timestamp() as usize;

    let claims = RefreshClaims {
        sub: user_id,
        exp,
        iat,
    };

    encode(&Header::new(Algorithm::HS512), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)
}
