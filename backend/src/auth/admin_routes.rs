use axum::Router;
use axum::routing::{post, put};

use crate::auth::admin_handlers::{
    create_series_handler, update_series_handler,
};
use crate::builder::startup::AppState;

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/series", post(create_series_handler))
        .route("/series/{id}", put(update_series_handler))
}
