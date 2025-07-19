use axum::Router;
use axum::routing::{post, put};

use crate::auth::admin_handlers::{
    create_manga_series_handler, update_manga_series_handler,
};
use crate::builder::startup::AppState;

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/series", post(create_manga_series_handler))
        .route("/series/{id}", put(update_manga_series_handler))
}
