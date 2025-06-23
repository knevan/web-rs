use crate::auth::admin_handlers::{create_series_handler, update_series_handler};
use axum::Router;
use axum::routing::post;
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn admin_routes() -> Router<Arc<Mutex<Connection>>> {
    Router::new()
        .route("/series", post(create_series_handler))
        .route("/series/:id", post(update_series_handler))
}
