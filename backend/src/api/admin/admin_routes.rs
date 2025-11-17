use axum::routing::{delete, get, patch, post};
use axum::Router;

use crate::api::admin::admin_series_handlers::{
    create_category_tag_handler, create_new_series_handler, delete_category_tag_handler,
    delete_series_handler, get_all_paginated_series_handler, get_list_category_tags_handler,
    get_series_category_tags_handler, repair_chapter_handler, update_existing_series_handler,
    upload_series_cover_image_handler,
};
use crate::api::admin::admin_user_handler::{
    delete_user_handler, get_all_paginated_users_handler, update_user_metadata_handler,
};
use crate::builder::startup::AppState;

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        // User management routes
        .route("/users/delete", delete(delete_user_handler))
        .route("/users/update", patch(update_user_metadata_handler))
        .route(
            "/users/paginated/list-search",
            get(get_all_paginated_users_handler),
        )
        // Series management routes
        .route("/series/add", post(create_new_series_handler))
        .route("/series/delete/{id}", delete(delete_series_handler))
        .route("/series/repair/chapter/{id}", post(repair_chapter_handler))
        .route(
            "/series/paginated/list-search",
            get(get_all_paginated_series_handler),
        )
        .route("/series/update/{id}", patch(update_existing_series_handler))
        .route("/series/tags/{id}", get(get_series_category_tags_handler))
        // Image upload routes
        .route(
            "/series/cover/upload/image",
            post(upload_series_cover_image_handler),
        )
        // Category Tag management routes
        .route("/category/tag/add", post(create_category_tag_handler))
        .route(
            "/category/tag/delete/{id}",
            delete(delete_category_tag_handler),
        )
        .route("/category/tag/list", get(get_list_category_tags_handler))
}
