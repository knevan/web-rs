use crate::common::jwt::Claims;
use crate::db::db;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum_core::response::{IntoResponse, Response};
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct SeriesRequest {
    title: String,
    description: Option<String>,
    cover_image_url: Option<String>,
    source_url: Option<String>,
    check_interval_minutes: i32,
}

/// Handler to create a new manhwa series.
/// This route is protected and can only be accessed by a logged-in admin.
pub async fn create_series_handler(
    claims: Claims,
    State(db_conn): State<Arc<Mutex<Connection>>>,
    Json(payload): Json<SeriesRequest>,
) -> Response {
    println!(
        "->> {:<12} - create_series_handler - user: {}",
        "Handler", claims.sub
    );

    let conn = db_conn.lock().await;

    match db::add_manga_series(
        &conn,
        &payload.title,
        payload.description.as_deref(),
        payload.cover_image_url.as_deref(),
        payload.source_url.as_deref(),
        payload.check_interval_minutes,
    ) {
        Ok(new_id) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"status": "success", "id": new_id})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"status": "error", "message": e.to_string()})),
        )
            .into_response(),
    }
}

/// Handler to update an existing manhwa series.
/// This route is also protected.
pub async fn update_series_handler(
    claims: Claims,
    State(db_conn): State<Arc<Mutex<Connection>>>,
    Path(series_id): Path<i32>,
    Json(payload): Json<SeriesRequest>,
) -> Response {
    println!(
        "->> {:<12} - update_series_handler - user: {}, series_id: {}",
        "HANDLER", claims.sub, series_id
    );

    let conn = db_conn.lock().await;

    match db::update_manga_series(
        &conn,
        series_id,
        &payload.title,
        payload.description.as_deref(),
        payload.cover_image_url.as_deref(),
        payload.source_url.as_deref(),
        payload.check_interval_minutes,
    ) {
        Ok(rows_affected) if rows_affected > 0 => {
            (
                StatusCode::OK,
                Json(serde_json::json!({"status": "success", "message": format!("Series {} updated", series_id)})),
            ).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"status": "error", "message": format!("Series with id {} not found", series_id)}))
            ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"status": "error", "message": e.to_string()}))
            ).into_response()
    }
}
