use crate::common::error::AuthError;
use axum::{RequestPartsExt, extract::FromRequestParts, http::request::Parts};
use axum_extra::TypedHeader;
use axum_extra::headers::{Authorization, authorization::Bearer};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::LazyLock;

/// Secret key for JWT signing and encryption
/// Load from environment variables do not hardcode sensitive information
static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret_key = env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");
    Keys::new(secret_key.as_bytes())
});

/// Keys is a struct that holds the encoding and decoding keys for JWT.
pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

/// The Keys struct is used to create the encoding and decoding keys for JWT.
impl Keys {
    fn new(secret_key: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret_key),
            decoding: DecodingKey::from_secret(secret_key),
        }
    }
}

/// Claims is a struct that represents the claims in the JWT token.
/// It contains the subject (user ID), expiration time, and issued at time.
/// The `Claims` struct is used to encode and decode the JWT tokens.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub exp: usize,  // Expiration time
    pub iat: usize,  // Issued at time
}

/// Custom extractor to get claims from requests.
/// This will be used in the protected handler to get
/// authenticated user information.
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract token from Authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

/// Create jwt token for a given user ID.
pub fn create_jwt(user_id: String) -> Result<String, AuthError> {
    let now = Utc::now();
    let iat = now.timestamp() as usize;

    let exp = (now + Duration::days(5)).timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        exp,
        iat,
    };

    encode(&Header::default(), &claims, &KEYS.encoding).map_err(|_| AuthError::TokenCreation)
}
