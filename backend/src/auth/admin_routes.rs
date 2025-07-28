use axum::Router;
use axum::routing::{delete, get, patch, post};

use crate::auth::admin_handlers::{
    create_new_series_handler, delete_series_handler,
    get_all_manga_series_handler, repair_chapter_handler,
    update_existing_series_handler, upload_series_cover_image_handler,
};
use crate::builder::startup::AppState;

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/series/add", post(create_new_series_handler))
        .route("/series/update/{id}", patch(update_existing_series_handler))
        .route("/series/list", get(get_all_manga_series_handler))
        .route("/upload/image", post(upload_series_cover_image_handler))
        .route("/series/delete/{id}", delete(delete_series_handler))
        .route("/series/repair/chapter/{id}", post(repair_chapter_handler))
}
