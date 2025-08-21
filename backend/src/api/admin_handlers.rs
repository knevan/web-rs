use crate::api::extractor::AdminUser;
use crate::builder::startup::AppState;
use crate::database::{NewSeriesData, Series, UpdateSeriesData};
use crate::task_workers::repair_chapter_worker;
use crate::task_workers::series_check_worker::SeriesCheckJob;
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
    category_ids: Vec<i32>,
}

// Admin endpoint to create new series
pub async fn create_new_series_handler(
    admin: AdminUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateSeriesRequest>,
) -> Response {
    let db_service = &state.db_service;

    println!(
        "->> {:<12} - create_series_handler - user: {}",
        "Handler", admin.0.username
    );

    // Random time to check target website
    let check_interval_minutes = rand::rng().random_range(90..=120);

    let new_series_data = NewSeriesData {
        title: &payload.title,
        original_title: payload.original_title.as_deref(),
        authors: payload.authors.as_ref(),
        category_ids: Some(&payload.category_ids),
        description: &payload.description,
        cover_image_url: &payload.cover_image_url,
        source_url: &payload.source_url,
        check_interval_minutes,
    };

    // Create new series in DB
    let new_series_id = match db_service.add_new_series(&new_series_data).await
    {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": e.to_string()}))
            )
                .into_response();
        }
    };

    let fetch_new_series: Series = match db_service
        .get_manga_series_by_id(new_series_id)
        .await
    {
        Ok(Some(series)) => series,
        _ => {
            eprintln!("Error fetching new series from DB: {}", new_series_id);
            return (
                StatusCode::CREATED,
                Json(serde_json::json!({"status": "success", "id": new_series_id, "warning": "Could not schedule immediate check."}))
            )
                .into_response();
        }
    };

    // Crate and send job to worker via priority queue
    let job = SeriesCheckJob {
        series: fetch_new_series,
    };
    if let Err(e) = state.worker_channels.series_check_tx.send(job).await {
        eprintln!(
            "Failed to send job to worker for series: {} {}",
            new_series_id, e
        );
    } else {
        println!(
            "Successfully scheduled immediate check for series: {}",
            new_series_id
        );
    }

    (
        StatusCode::CREATED,
        Json(serde_json::json!({"status": "success", "id": new_series_id, "message": "Series created and scheduled for immediate scraping"})),
    )
        .into_response()
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
    category_ids: Option<Vec<i32>>,
}

pub async fn update_existing_series_handler(
    Path(series_id): Path<i32>,
    admin: AdminUser,
    State(state): State<AppState>,
    Json(payload): Json<UpdateSeriesRequest>,
) -> Response {
    let db_service = &state.db_service;

    println!(
        "->> {:<12} - update_series_handler - user: {}, series_id: {}",
        "HANDLER", admin.0.username, series_id
    );

    let update_series_data = UpdateSeriesData {
        title: payload.title.as_deref(),
        original_title: payload.original_title.as_deref(),
        authors: payload.authors.as_ref(),
        description: payload.description.as_deref(),
        cover_image_url: payload.cover_image_url.as_deref(),
        source_url: payload.source_url.as_deref(),
        check_interval_minutes: None,
        category_ids: payload.category_ids.as_deref(),
    };

    // Call the async method on the DatabaseService instance
    match db_service
        .update_series_metadata(series_id, &update_series_data)
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
    #[serde(default)]
    search: Option<String>,
}

fn default_page() -> u32 {
    1
}
fn default_page_size() -> u32 {
    25
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesResponse {
    id: i32,
    title: String,
    original_title: Option<String>,
    description: String,
    cover_image_url: String,
    source_url: String,
    authors: Vec<String>,
    last_updated: String,
    processing_status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedSeriesResponse {
    items: Vec<SeriesResponse>,
    total_items: i64,
}

pub async fn get_all_manga_series_handler(
    admin: AdminUser,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
) -> Response {
    println!(
        "->> {:<12} - get_all_manga_series_handler - user: {}",
        "HANDLER", admin.0.username
    );

    match state
        .db_service
        .get_admin_paginated_series(pagination.page, pagination.page_size, pagination.search.as_deref())
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
                    processing_status: s.processing_status.to_string(),
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

#[derive(Deserialize)]
pub struct RepairChapterRequest {
    pub chapter_number: f32,
    pub new_chapter_url: String,
    pub new_chapter_title: Option<String>,
}

pub async fn repair_chapter_handler(
    admin: AdminUser,
    Path(series_id): Path<i32>,
    State(state): State<AppState>,
    Json(payload): Json<RepairChapterRequest>,
) -> Response {
    println!(
        "->> {:<12} - repair_chapter_handler - user: {}, series_id: {}",
        "HANDLER", admin.0.username, series_id
    );

    let repair_chapter_msg = repair_chapter_worker::RepairChapterMsg {
        series_id,
        chapter_number: payload.chapter_number,
        new_chapter_url: payload.new_chapter_url,
        new_chapter_title: payload.new_chapter_title,
    };

    match state.worker_channels.repair_tx.send(repair_chapter_msg).await {
        Ok(_) => {
            (
                StatusCode::ACCEPTED,
                Json(serde_json::json!({"status": "success", "message": "Chapter has been scheduled for repair"})),
            )
                .into_response()
        }
        Err(_) => {
            // This error occurs if the worker has crashed and the channel is closed.
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Could not schedule repair. The repair service may be down."})),
            )
                .into_response()
        }
    }
}

pub async fn delete_series_handler(
    admin: AdminUser,
    Path(series_id): Path<i32>,
    State(state): State<AppState>,
) -> Response {
    println!(
        "->> {:<12} - SCHEDULE DELETE - user: {}, series_id: {}",
        "HANDLER", admin.0.username, series_id
    );

    // Send series_id to deletion worker with channel
    match state.db_service.mark_series_for_deletion(series_id).await {
        Ok(row_affected) if row_affected > 0 => {
            (
                StatusCode::ACCEPTED,
                Json(serde_json::json!({"status": "success", "message": "Series has been scheduled for deletion."})),
            )
                .into_response()
        }
        Ok(_) => {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"status": "error", "message": "Series not found or already pending deletion."})),
            )
                .into_response()
        }
        Err(_) => {
            // This error accours if the worker crashes and the channel is closed
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Could not schedule deletion. The deletion service may be down."})),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct CreateCategoryTagRequest {
    pub name: String,
}

pub async fn create_category_tag_handler(
    admin: AdminUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateCategoryTagRequest>,
) -> Response {
    println!(
        "->> {:<12} - create_category_tag_handler - user: {}",
        "HANDLER", admin.0.username
    );

    match state.db_service.create_category_tag(&payload.name).await {
        Ok(new_category) => {
            (StatusCode::CREATED, Json(serde_json::json!({"status": "success", "category": new_category})),
            )
                .into_response()
        }
        Err(e) => {
            // Check for unique violation error from PostgreSQL (code 23505)
            if let Some(sqlx::Error::Database(db_error)) =
                e.root_cause().downcast_ref::<sqlx::Error>()
                && db_error.code() == Some(std::borrow::Cow::from("23505")) {
                    return (
                            StatusCode::CONFLICT,
                            Json(serde_json::json!({"status": "error", "message": "Category tag already exists."})),
                        )
                            .into_response();
                }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub async fn delete_category_tag_handler(
    admin: AdminUser,
    State(state): State<AppState>,
    Path(category_id): Path<i32>,
) -> Response {
    println!(
        "->> {:<12} - delete_category_tag_handler - user: {}, category_id: {}",
        "HANDLER", admin.0.username, category_id
    );

    match state.db_service.delete_category_tag(category_id).await {
        Ok(row_affected) if row_affected > 0 => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "success", "message": "Category tag has been deleted."})),
        )
            .into_response(),
        Ok(_) => ( // row_affected is 0
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"status": "error", "message": "Category tag not found."})),
        )
            .into_response(),
        Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": e.to_string()})),
            )
                .into_response(),
    }
}

pub async fn get_list_category_tags_handler(
    admin: AdminUser,
    State(state): State<AppState>,
) -> Response {
    println!(
        "->> {:<12} - get_list_category_tags_handler - user: {}",
        "HANDLER", admin.0.username
    );

    match state.db_service.get_list_all_categories().await {
        Ok(categories) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "success", "categories": categories})),
        )
            .into_response(),
        Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_series_category_tags_handler(
    admin: AdminUser,
    State(state): State<AppState>,
    Path(series_id): Path<i32>,
) -> Response {
    println!(
        "->> {:<12} - get_series_category_tags_handler - user: {}, series_id: {}",
        "HANDLER", admin.0.username, series_id
    );

    match state.db_service.get_category_tag_by_series_id(series_id).await {
        Ok(tags) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "success", "tags": tags})),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"status": "error", "message": e.to_string()}))
        ).into_response(),
    }
}
