use crate::auth::admin_router::admin_routes;
use crate::auth::handlers::{
    login_handler, logout_handler, protected_handler, refresh_token_handler,
};
use axum::Router;
use axum::routing::{get, post};
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn routes() -> Router<Arc<Mutex<Connection>>> {
    // Router for admin, protected inside handler
    let admin_api_routes = Router::new().nest("/admin", admin_routes());

    // Router for public auth
    let auth_api_routes = Router::new()
        .route("/login", post(login_handler))
        .route("/refresh", get(refresh_token_handler))
        .route("/logout", post(logout_handler))
        .route("/user", post(protected_handler));

    // Combine both routers under prefix "/api"
    Router::new()
        .nest("/api/auth", auth_api_routes)
        .nest("/api", admin_api_routes)
}
