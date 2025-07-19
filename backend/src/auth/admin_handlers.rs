use crate::builder::startup::AppState;
use crate::common::jwt::Claims;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum_core::response::{IntoResponse, Response};
use axum_extra::extract::Multipart;
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateSeriesRequest {
    title: String,
    original_title: Option<String>,
    authors: Option<Vec<String>>,
    description: String,
    cover_image_url: String,
    source_url: String,
}

// This route is protected and can only be accessed by a logged-in admin-dashboard.
pub async fn create_manga_series_handler(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateSeriesRequest>,
) -> Response {
    let db_service = &state.db_service;

    println!(
        "->> {:<12} - create_series_handler - user: {}",
        "Handler", claims.sub
    );

    let check_interval_minutes = rand::rng().random_range(100..=150);

    match db_service
        .add_new_manga_series(
            &payload.title,
            payload.original_title.as_deref(),
            payload.authors.as_ref(),
            &payload.description,
            &payload.cover_image_url,
            &payload.source_url,
            check_interval_minutes
        )
        .await
    {
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

#[derive(Deserialize)]
pub struct UpdateSeriesRequest {
    title: Option<String>,
    original_title: Option<String>,
    description: Option<String>,
    cover_image_url: Option<String>,
    source_url: Option<String>,
}

pub async fn update_manga_series_handler(
    Path(series_id): Path<i32>,
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<UpdateSeriesRequest>,
) -> Response {
    let db_service = &state.db_service;

    println!(
        "->> {:<12} - update_series_handler - user: {}, series_id: {}",
        "HANDLER", claims.sub, series_id
    );

    // Call the async method on the DatabaseService instance
    match db_service
        .update_manga_series_metadata(
            series_id,
            payload.title.as_deref(),
            payload.original_title.as_deref(),
            payload.description.as_deref(),
            payload.cover_image_url.as_deref(),
            payload.source_url.as_deref(),
            None,
        )
        .await
    {
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

#[derive(Serialize)]
pub struct UploadResponse {
    status: String,
    url: String,
}

pub async fn upload_series_cover_image_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Response {
    if let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field.file_name().unwrap_or("unknown_file").to_string();
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let file_data = match field.bytes().await {
            Ok(bytes) => bytes.to_vec(),
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"status": "error", "message": format!("Failed to read file bytes: {}", e)}))
                ).into_response();
            }
        };

        let file_extension = std::path::Path::new(&file_name)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("jpg");

        let unique_image_key =
            format!("cover-manga/{}.{}", Uuid::new_v4(), file_extension);

        return match state
            .storage_client
            .upload_image_file(file_data, &unique_image_key, &content_type)
            .await
        {
            Ok(url) => (
                StatusCode::OK,
                Json(UploadResponse {
                    status: "success".to_string(),
                    url,
                }),
            )
                .into_response(),
            Err(e) => {
                eprintln!("Failed to upload cover image: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"status": "error", "message": "Failed to upload cover image to storage"}))
                ).into_response()
            }
        };
    }

    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"status": "error", "message": "No cover image file found"}))
    ).into_response()
}
