use crate::builder::startup::AppState;
use crate::common::jwt::Claims;
use crate::database::{NewSeriesData, UpdateSeriesData};
use axum::Json;
use axum::extract::{Path, Query, State};
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

    let new_series_data = NewSeriesData {
        title: &payload.title,
        original_title: payload.original_title.as_deref(),
        authors: payload.authors.as_ref(),
        description: &payload.description,
        cover_image_url: &payload.cover_image_url,
        source_url: &payload.source_url,
        check_interval_minutes,
    };

    match db_service.add_new_series(&new_series_data)
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
#[serde(rename_all = "camelCase")]
pub struct UpdateSeriesRequest {
    title: Option<String>,
    original_title: Option<String>,
    authors: Option<Vec<String>>,
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

    let update_series_data = UpdateSeriesData {
        title: payload.title.as_deref(),
        original_title: payload.original_title.as_deref(),
        authors: payload.authors.as_ref(),
        description: payload.description.as_deref(),
        cover_image_url: payload.cover_image_url.as_deref(),
        source_url: payload.source_url.as_deref(),
        check_interval_minutes: None,
    };

    // Call the async method on the DatabaseService instance
    match db_service
        .update_manga_series_metadata(series_id, &update_series_data)
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
            .upload_cover_image_file(
                file_data,
                &unique_image_key,
                &content_type,
            )
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

#[derive(Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_page_size")]
    page_size: u32,
}

fn default_page() -> u32 {
    1
}
fn default_page_size() -> u32 {
    25
}
#[derive(Serialize)]
pub struct SeriesResponse {
    id: i32,
    title: String,
    original_title: String,
    description: String,
    cover_image_url: String,
    source_url: String,
    authors: Vec<String>,
    last_updated: String,
}

#[derive(Serialize)]
pub struct PaginatedSeriesResponse {
    items: Vec<SeriesResponse>,
    total_items: i64,
}

pub async fn get_all_manga_series_handler(
    claims: Claims,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
) -> Response {
    println!(
        "->> {:<12} - get_all_manga_series_handler - user: {}",
        "HANDLER", claims.sub
    );

    match state
        .db_service
        .get_paginated_series_with_authors(pagination.page, pagination.page_size)
        .await
    {
        Ok(paginated_result) => {
            let response_series_items: Vec<SeriesResponse> = paginated_result
                .items
                .into_iter()
                .map(|s| SeriesResponse {
                    id: s.id,
                    title: s.title,
                    original_title: s.original_title,
                    description: s.description,
                    cover_image_url: s.cover_image_url,
                    source_url: s.current_source_url,
                    authors: serde_json::from_value(s.authors).unwrap_or_else(|_| vec![]),
                    last_updated: s.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                })
                .collect();

            let response_series_data = PaginatedSeriesResponse {
                items: response_series_items,
                total_items: paginated_result.total_items,
            };

            (StatusCode::OK, Json(response_series_data)).into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": e.to_string()})),
            )
                .into_response()
        }
    }
}

fn extract_object_key_from_url(url: &str, base_url: &str) -> Option<String> {
    // Ensure base url does not have trailing slash
    let trimmed_base_url = base_url.trim_end_matches('/');
    if url.starts_with(trimmed_base_url) {
        // Remove base url(cdn url) and leading slash to get object key
        Some(
            url[trimmed_base_url.len()..]
                .trim_start_matches('/')
                .to_string(),
        )
    } else {
        None
    }
}

pub async fn delete_series_handler(
    Path(series_id): Path<i32>,
    claims: Claims,
    State(state): State<AppState>,
) -> Response {
    println!(
        "->> {:<12} - delete_series_handler - user: {}, series_id: {}",
        "HANDLER", claims.sub, series_id
    );

    let db_service = &state.db_service;
    let storage_client = &state.storage_client;

    // Get all image urls from DB and verify series exist
    let image_data = match db_service
        .get_image_keys_for_series_deletion(series_id)
        .await
    {
        Ok(Some(data)) => data,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"status": "error", "message": format!("Series with id {} not found", series_id)}),
                ),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": format!("Failed to retrieve data for deletion: {}", e)}),
                ),
            )
                .into_response()
        }
    };

    // Collect object key and delete from storage
    let mut keys_to_delete_in_storage: Vec<String> = Vec::new();
    let domain_cdn_url = storage_client.domain_cdn_url();

    if let Some(cover_series_url) = image_data.cover_image_url {
        if let Some(key) =
            extract_object_key_from_url(&cover_series_url, domain_cdn_url)
        {
            keys_to_delete_in_storage.push(key);
        } else {
            eprintln!(
                "Could not parse key from cover series url: {}",
                cover_series_url
            );
        }
    }

    for chapter_image_url in image_data.chapter_image_urls {
        if let Some(key) =
            extract_object_key_from_url(&chapter_image_url, domain_cdn_url)
        {
            keys_to_delete_in_storage.push(key);
        } else {
            eprintln!(
                "Could not parse key from chapter image url: {}",
                chapter_image_url
            );
        }
    }

    if !keys_to_delete_in_storage.is_empty() {
        if let Err(e) = storage_client
            .delete_image_objects(keys_to_delete_in_storage)
            .await
        {
            // If storage deletion fails, stop immediately to allow retry
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": format!("Failed to delete images from storage: {}. DB not modified", e)})),
                )
                .into_response();
        }
    }

    // Delete series and all related records(data) from DB
    match db_service.delete_series_by_id(series_id).await {
        Ok(row_affected) if row_affected > 0 => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "success", "message": format!("Series {} and all related data deleted successfully", series_id)})),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"status": "error", "message": format!("Series {} not found for final deletion", series_id)})),
        )
            .into_response(),
        Err(e) => {
            // CRITICAL: Files are gone from storage, but DB records remain. Manual intervention is required.
            eprintln!("CRITICAL ERROR: Files for series {} deleted from storage, but DB deletion failed: {}", series_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": format!("CRITICAL: DB deletion failed after storage cleanup. Manual intervention required for series_id {}. Error: {}", series_id, e)})),
            )
                .into_response()
        }
    }
}
