use crate::auth::handlers::{login_handler, protected_handler};
use axum::Router;
use axum::routing::{get, post};

pub fn routes() -> Router {
    Router::new()
        .route("/api/admin/login", post(login_handler))
        .route("/api/admin/users", get(protected_handler))
}
