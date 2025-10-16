use axum::Router;
use axum::routing::{get, patch, post};

use crate::api::admin_routes::admin_routes;
use crate::api::auth_handlers::{
    forgot_password_handler, login_handler, logout_handler, protected_handler,
    realtime_check_username_handler, refresh_token_handler, register_new_user_handler,
    reset_password_handler,
};
use crate::api::series_handlers::{
    browse_series_handler, fetch_chapter_details_handler,
    fetch_most_viewed_series_handler, fetch_new_series_handler,
    fetch_series_details_by_id_handler, fetch_updated_series_chapter_handler,
    get_all_categories_handler, get_chapter_comment_handler, get_series_comment_handler,
    post_chapter_comment_handler, post_series_comment_handler, rate_series_handler,
    record_series_view_handler, update_existing_comment_handler,
    upload_comment_attachments_handler, vote_on_comment_handler,
};
use crate::api::user_handlers::{
    add_bookmark_series_handler, delete_bookmark_series_handler,
    get_bookmark_status_current_user_handler, get_user_bookmark_library_handler,
    get_user_profile_handler, update_user_avatar_handler,
    update_user_password_setting_handler, update_user_profile_handler,
};
use crate::builder::startup::AppState;

pub fn routes() -> Router<AppState> {
    // Route for user auth api
    let auth_api_routes = Router::new()
        .route("/login", post(login_handler))
        .route("/register", post(register_new_user_handler))
        .route("/refresh", post(refresh_token_handler))
        .route("/logout", post(logout_handler))
        .route("/user", post(protected_handler))
        .route("/check-username", post(realtime_check_username_handler))
        .route("/forgot-password", post(forgot_password_handler))
        .route("/reset-password", post(reset_password_handler));

    // Router user related api
    let user_api = Router::new()
        .route("/user/bookmark", get(get_user_bookmark_library_handler))
        .route(
            "/user/profile",
            get(get_user_profile_handler).patch(update_user_profile_handler),
        )
        .route(
            "/user/profile/password",
            patch(update_user_password_setting_handler),
        )
        .route("/user/profile/avatar", post(update_user_avatar_handler));

    // Route for public api
    let public_series_api_routes = Router::new()
        .route("/series/most-viewed", get(fetch_most_viewed_series_handler))
        .route("/series/new-series", get(fetch_new_series_handler))
        .route(
            "/series/latest-updated-series",
            get(fetch_updated_series_chapter_handler),
        )
        .route("/series/tags", get(get_all_categories_handler))
        .route("/series/browse", get(browse_series_handler))
        .route(
            "/series/details/{id}",
            get(fetch_series_details_by_id_handler),
        )
        .route(
            "/series/{id}/chapter/{chapter_number}",
            get(fetch_chapter_details_handler),
        )
        .route("/series/{id}/rate", post(rate_series_handler))
        .route("/series/{id}/views-count", post(record_series_view_handler))
        .route(
            "/series/{id}/bookmark",
            post(add_bookmark_series_handler).delete(delete_bookmark_series_handler),
        )
        .route(
            "/series/{id}/bookmark/status",
            get(get_bookmark_status_current_user_handler),
        );

    // Router comments related api
    let comments_api = Router::new()
        .route(
            "/series/{id}/comments",
            get(get_series_comment_handler).post(post_series_comment_handler),
        )
        .route(
            "/series/chapter/{id}/comments",
            get(get_chapter_comment_handler).post(post_chapter_comment_handler),
        )
        .route(
            "/comments/{id}/edit",
            patch(update_existing_comment_handler),
        )
        .route("/comments/{id}/vote", post(vote_on_comment_handler))
        .route(
            "/comments/attachments/upload",
            post(upload_comment_attachments_handler),
        );

    // Merge same prefix "/api" routes into one
    let general_api = public_series_api_routes.merge(user_api).merge(comments_api);

    // Combine routers under prefix "/api"
    Router::new()
        .nest("/api/admin", admin_routes())
        .nest("/api/auth", auth_api_routes)
        .nest("/api", general_api)
}
