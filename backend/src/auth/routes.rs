use axum::Router;
use axum::routing::{get, post};

use crate::auth::admin_router::admin_routes;
use crate::auth::handlers::{
    check_username_handler, login_handler, logout_handler, protected_handler,
    refresh_token_handler, register_user_handler,
};
use crate::db::db::DatabaseService;

pub fn routes() -> Router<DatabaseService> {
    // Router for admin-dashboard, protected inside handler
    let admin_api_routes =
        Router::new().nest("/admin-dashboard", admin_routes());

    // Router for public auth
    let auth_api_routes = Router::new()
        .route("/login", post(login_handler))
        .route("/register", post(register_user_handler))
        .route("/refresh", get(refresh_token_handler))
        .route("/logout", post(logout_handler))
        .route("/user", post(protected_handler))
        .route("/check-username", post(check_username_handler));

    // Combine both routers under prefix "/api"
    // The structure is flat, avoiding double nesting.
    Router::new()
        .nest("/api/auth", auth_api_routes)
        .nest("/api/admin-dashboard", admin_api_routes)
}
