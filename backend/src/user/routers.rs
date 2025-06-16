use crate::user::handlers::{get_manga_updates, manga_updates_handler};
use axum::{Router, routing::get};
use std::sync::Arc;
use tokio_rusqlite_new::Connection;

pub struct AppState {
    pub db: Arc<Connection>,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/mangaupdates", get(manga_updates_handler))
        .route("/api/mangaupdates", get(get_manga_updates))
        .with_state(state)
}
