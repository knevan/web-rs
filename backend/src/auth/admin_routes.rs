use axum::Router;
use axum::routing::{get, post, put};

use crate::auth::admin_handlers::{
    create_manga_series_handler, get_all_manga_series_handler,
    update_manga_series_handler, upload_series_cover_image_handler,
};
use crate::builder::startup::AppState;

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/series/add", post(create_manga_series_handler))
        .route("/series/update/{id}", put(update_manga_series_handler))
        .route("/series/list", get(get_all_manga_series_handler))
        .route("/upload/image", post(upload_series_cover_image_handler))
}
