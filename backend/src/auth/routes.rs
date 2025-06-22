use crate::auth::handlers::{
    login_handler, logout_handler, protected_handler, refresh_token_handler,
};
use axum::Router;
use axum::routing::{get, post};

pub fn routes() -> Router {
    Router::new()
        .route("/login", post(login_handler))
        .route("/refresh", get(refresh_token_handler))
        .route("/logout", post(logout_handler))
        .route("/user", post(protected_handler))
}
