use crate::api::extractor::{AuthenticatedUser, OptionalAuthenticatedUser};
use crate::builder::startup::AppState;
use crate::database::{
    CategoryTag, CommentEntityType, NewCommentPayload, Series, SeriesChapter,
    SeriesOrderBy,
};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum_core::__private::tracing::error;
use axum_core::response::{IntoResponse, Response};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct MostViewedParams {
    #[serde(default = "default_period")]
    period: String,
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_period() -> String {
    "week".to_string()
}

fn default_limit() -> i64 {
    20
}

pub async fn fetch_most_viewed_series_handler(
    State(state): State<AppState>,
    Query(params): Query<MostViewedParams>,
) -> Response {
    // Map the user-friendly period string
    let period_str = match params.period.to_lowercase().as_str() {
        "hour" => "1 hour",
        "day" => "1 day",
        "week" => "1 week",
        "month" => "1 month",
        _ => "1 days",
    };

    match state
        .db_service
        .fetch_most_viewed_series(period_str, params.limit)
        .await
    {
        Ok(series) => (StatusCode::OK, Json(series)).into_response(),
        Err(e) => {
            eprintln!("Error fetching most viewed series: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Could not retrieve most viewed series."})),
            ).into_response()
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesDataResponse {
    series: Series,
    chapters: Vec<SeriesChapter>,
    authors: Vec<String>,
    category_tags: Vec<CategoryTag>,
}

// Fetch all details for a single series
pub async fn fetch_series_details_by_id_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Response {
    let db = &state.db_service;

    let series = match db.get_series_by_id(id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"status": "error", "message": "Series not found."})),
            )
                .into_response();
        }
        Err(e) => {
            error!("Error fetching series details: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Could not retrieve series details."})),
            )
                .into_response();
        }
    };

    let series_id = series.id;

    // Fetch authors, chapters, and categories tag in parallel
    let (authors_result, chapters_result, categories_result) = tokio::join!(
        db.get_authors_by_series_id(series_id),
        db.get_chapters_by_series_id(series_id),
        db.get_category_tag_by_series_id(series_id),
    );

    let response_data = SeriesDataResponse {
        series,
        authors: authors_result.unwrap_or_default(),
        chapters: chapters_result.unwrap_or_default(),
        category_tags: categories_result.unwrap_or_default(),
    };

    (StatusCode::OK, Json(response_data)).into_response()
}

pub async fn fetch_new_series_handler(
    State(state): State<AppState>,
) -> Response {
    match state
        .db_service
        .get_public_series_paginated(1, 20, SeriesOrderBy::CreatedAt)
        .await
    {
        Ok(series) => (StatusCode::OK, Json(series)).into_response(),
        Err(e) => {
            eprintln!("Error fetching new series: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Could not retrieve new series."})),
            ).into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_pagesize")]
    page_size: u32,
    #[serde(default)]
    search: Option<String>,
}

fn default_page() -> u32 {
    1
}
fn default_pagesize() -> u32 {
    50
}

pub async fn fetch_updated_series_chapter_handler(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Response {
    match state
        .db_service
        .get_latest_release_series_chapter_paginated(
            params.page,
            params.page_size,
        )
        .await
    {
        Ok(paginated_result) => {
            (StatusCode::OK, Json(paginated_result)).into_response()
        }
        Err(e) => {
            error!("Error fetching updated series: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Could not retrieve updated series."})),
            )
                .into_response()
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterDetailsResponse {
    series_title: String,
    chapter_title: Option<String>,
    chapter_number: f32,
    pages: Vec<String>,
    all_chapters: Vec<SeriesChapter>,
    prev_chapter_number: Option<f32>,
    next_chapter_number: Option<f32>,
}

pub async fn fetch_chapter_details_handler(
    State(state): State<AppState>,
    Path((series_id, chapter_number)): Path<(i32, f32)>,
) -> Response {
    println!(
        "->> {:<12} - fetch_chapter_images - series_id: {}, chapter: {}",
        "HANDLER", series_id, chapter_number
    );

    let db = &state.db_service;
    let base_url = state.storage_client.domain_cdn_url();

    let (series_result, all_chapters_result, images_result) = tokio::join!(
        db.get_series_by_id(series_id),
        db.get_chapters_by_series_id(series_id),
        db.get_images_urls_for_chapter_series(series_id, chapter_number),
    );

    // Get series title
    let series = match series_result {
        Ok(Some(s)) => s,
        _ => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"status": "error", "message": "Series not found."})),
        ).into_response(),
    };

    // Get chapter images list
    let object_keys = match images_result {
        Ok(img_chap) => img_chap,
        Err(e) => {
            error!("Error fetching chapter images: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Could not retrieve chapter images."})),
            ).into_response();
        }
    };

    let pages = object_keys
        .into_iter()
        .map(|key| format!("{}/{}", base_url, key))
        .collect();

    // Get all chapters for the series and find current, next and previous chapters
    let all_chapters = match all_chapters_result {
        Ok(mut chaps) => {
            chaps.sort_by(|a, b| {
                a.chapter_number.partial_cmp(&b.chapter_number).unwrap()
            });
            chaps
        }
        Err(e) => {
            error!("Error fetching chapter list: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Could not retrieve chapter list."})),
            ).into_response();
        }
    };

    let current_chapter_idx = all_chapters
        .iter()
        .position(|c| c.chapter_number == chapter_number);

    let current_chapter_index = match current_chapter_idx {
        Some(index) => index,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"status": "error", "message": "Chapter not found in this series."})),
            ).into_response();
        }
    };

    let current_chapter = &all_chapters[current_chapter_index];

    let prev_chapter_number = if current_chapter_index > 0 {
        all_chapters
            .get(current_chapter_index - 1)
            .map(|c| c.chapter_number)
    } else {
        None
    };

    let next_chapter_number = all_chapters
        .get(current_chapter_index + 1)
        .map(|c| c.chapter_number);

    let response_data = ChapterDetailsResponse {
        series_title: series.title,
        chapter_title: current_chapter.title.clone(),
        chapter_number,
        pages,
        all_chapters,
        prev_chapter_number,
        next_chapter_number,
    };

    (StatusCode::OK, Json(response_data)).into_response()
}

pub async fn record_series_view_handler(
    State(state): State<AppState>,
    Path(series_id): Path<i32>,
) -> Response {
    println!(
        "->> {:<12} - record_series_view - series_id: {}",
        "HANDLER", series_id
    );

    let db = &state.db_service;

    match db.record_series_view(series_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "success", "message": "View Recorded."})),
        )
            .into_response(),
        Err(e) => {
            error!("Error recording series view for id {}: {}", series_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Could not record series view."})),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct RateSeriesPayload {
    rating: i16,
}
#[derive(Serialize)]
pub struct RateSeriesResponse {
    message: String,
    new_total_score: i64,
    new_total_count: i32,
}

pub async fn rate_series_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(series_id): Path<i32>,
    Json(payload): Json<RateSeriesPayload>,
) -> Response {
    // validate rating value
    if !(1..=5).contains(&payload.rating) {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::json!({"error": "Rating must be between 1 and 5"}),
            ),
        )
            .into_response();
    }

    match state
        .db_service
        .add_or_update_series_rating(series_id, payload.rating, user.id)
        .await
    {
        Ok(_) => {
            match state.db_service.get_series_by_id(series_id).await {
                Ok(Some(series)) => {
                    let response = RateSeriesResponse {
                        message: "Rating submitted".to_string(),
                        new_total_score: series.total_rating_score,
                        new_total_count: series.total_ratings_count
                    };
                    (StatusCode::OK, Json(response)).into_response()
                }
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed for retrieve updated series data"})),
                    ).into_response(),
            }
        }
        Err(e) => {
            error!("Failed to proess rating: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to update rating: {}", e)})),
                ).into_response()
        }
    }
}

// Fetch comments
pub async fn get_series_comment_handler(
    State(state): State<AppState>,
    Path(series_id): Path<i32>,
    user: OptionalAuthenticatedUser,
) -> Response {
    let user_id = user.0.map(|u| u.id);
    match state
        .db_service
        .get_comments(CommentEntityType::Series, series_id, user_id)
        .await
    {
        Ok(comments) => (StatusCode::OK, Json(comments)).into_response(),
        Err(e) => {
            error!(
                "[SERIES] Failed to get comments for series {}: {}",
                series_id, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to get comments for series"})),
            ).into_response()
        }
    }
}

// Post new comment
pub async fn post_series_comment_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(series_id): Path<i32>,
    Json(payload): Json<NewCommentPayload>,
) -> Response {
    println!(
        "->> {:<12} - record_series_view - series_id: {:?}",
        "HANDLER", series_id
    );
    // Create the new comment and get its ID
    let new_comment_id = match state
        .db_service
        .create_new_comment(
            user.id,
            CommentEntityType::Series,
            series_id,
            &payload.content_markdown,
            payload.parent_id,
        )
        .await
    {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to create comment for series {}: {}", series_id, e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create comment for series"})),
            )
                .into_response();
        }
    };

    // After creating, fetch the full data for the new comment
    match state
        .db_service
        .get_comment_by_id(new_comment_id, Some(user.id))
        .await
    {
        // If fetch is successful, return the full comment object
        Ok(Some(new_comment)) => (StatusCode::OK, Json(new_comment)).into_response(),
        // Handle cases where the comment couldn't be fetched right after creation
        Ok(None) | Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Comment created but failed to retrieve its data"})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct UpdateCommentPayload {
    pub content_markdown: String,
}

pub async fn update_existing_comment_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(comment_id): Path<i64>,
    Json(payload): Json<UpdateCommentPayload>,
) -> Response {
    match state
        .db_service
        .update_existing_comment(comment_id, user.id, &payload.content_markdown)
        .await
    {
        Ok(Some(updated_html)) => {
            (
                StatusCode::OK,
                Json(serde_json::json!({"message": "Comment updated successfully", "new_html_content": updated_html})),
            ).into_response()
        }
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({"message": "Comment not found or permission denied"}))).into_response()
        }
        Err(e) => {
            error!(
                "Failed to update existing comment with id {}: {}",
                comment_id, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Could not update comment"})),
            )
                .into_response()
        }
    }
}
