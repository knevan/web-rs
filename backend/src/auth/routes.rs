use axum::Router;
use axum::routing::{get, post};

use crate::auth::admin_routes::admin_routes;
use crate::auth::handlers::{
    fetch_most_viewed_series_handler, forgot_password_handler, login_handler,
    logout_handler, protected_handler, realtime_check_username_handler,
    refresh_token_handler, register_new_user_handler, reset_password_handler,
};
use crate::builder::startup::AppState;

pub fn routes() -> Router<AppState> {
    // Router for public auth
    let auth_api_routes = Router::new()
        .route("/login", post(login_handler))
        .route("/register", post(register_new_user_handler))
        .route("/refresh", post(refresh_token_handler))
        .route("/logout", post(logout_handler))
        .route("/user", post(protected_handler))
        .route("/check-username", post(realtime_check_username_handler))
        .route("/forgot-password", post(forgot_password_handler))
        .route("/reset-password", post(reset_password_handler));

    // Router for most view updates
    let public_series_api_routes = Router::new()
        .route("/series/most-viewed", get(fetch_most_viewed_series_handler));

    // Combine both routers under prefix "/api"
    // The structure is flat, avoiding double nesting.
    Router::new()
        .nest("/api/auth", auth_api_routes)
        .nest("/api/admin", admin_routes())
        .nest("/api", public_series_api_routes)
}
