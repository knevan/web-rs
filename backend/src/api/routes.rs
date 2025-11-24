use axum::Router;

use crate::api::admin::admin_routes::admin_routes;
use crate::api::public::public_routes::{auth_api_routes, general_api_routes};
use crate::builder::startup::AppState;

// Merge all routes into one
pub fn merged_routes() -> Router<AppState> {
    // Combine routers under prefix "/api"
    Router::new()
        .nest("/api/admin", admin_routes())
        .nest("/api/auth", auth_api_routes())
        .nest("/api", general_api_routes())
}
