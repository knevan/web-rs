use axum::Json;
use axum::extract::{Path, Query, State};
use axum_core::__private::tracing::error;
use axum_core::response::{IntoResponse, Response};
use axum_extra::extract::Multipart;
use reqwest::StatusCode;
use serde::de::{Deserializer, Error};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::api::extractor::{AuthenticatedUser, OptionalAuthenticatedUser};
use crate::builder::startup::AppState;
use crate::database::{
    CategoryTag, Comment, CommentEntityType, Series, SeriesChapter, SeriesOrderBy,
    VotePayload,
};

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

pub async fn fetch_new_series_handler(State(state): State<AppState>) -> Response {
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
#[serde(rename_all = "camelCase")]
pub struct SeriesPaginationParams {
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_pagesize")]
    page_size: u32,
}

fn default_page() -> u32 {
    1
}
fn default_pagesize() -> u32 {
    50
}

pub async fn fetch_updated_series_chapter_handler(
    State(state): State<AppState>,
    Query(params): Query<SeriesPaginationParams>,
) -> Response {
    match state
        .db_service
        .get_latest_release_series_chapter_paginated(params.page, params.page_size)
        .await
    {
        Ok(paginated_result) => (StatusCode::OK, Json(paginated_result)).into_response(),
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
    chapter_id: i32,
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
        _ => return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"status": "error", "message": "Series not found."})),
        )
            .into_response(),
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
            chaps
                .sort_by(|a, b| a.chapter_number.partial_cmp(&b.chapter_number).unwrap());
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
        chapter_id: current_chapter.id,
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
            Json(serde_json::json!({"error": "Rating must be between 1 and 5"})),
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

// Fetch series comments
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
        Ok(mut comments) => {
            let base_url = state.storage_client.domain_cdn_url();

            let mut stack: Vec<&mut Comment> = comments.iter_mut().collect();
            while let Some(comment) = stack.pop() {
                if let Some(urls) = &mut comment.attachment_urls {
                    // Iterate over each URL string mutably.
                    for url in urls.iter_mut() {
                        // Prepend the base_url to the existing URL string.
                        *url = format!("{}/{}", base_url, url);
                    }
                }

                stack.extend(comment.replies.iter_mut());
            }

            (StatusCode::OK, Json(comments)).into_response()
        }
        Err(e) => {
            error!(
                "[SERIES] Failed to get comments for series {}: {}",
                series_id, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to get comments for series"})),
            )
                .into_response()
        }
    }
}

// Fetch chapter comments
pub async fn get_chapter_comment_handler(
    State(state): State<AppState>,
    Path(chapter_id): Path<i32>,
    user: OptionalAuthenticatedUser,
) -> Response {
    let user_id = user.0.map(|u| u.id);
    match state
        .db_service
        .get_comments(CommentEntityType::SeriesChapters, chapter_id, user_id)
        .await
    {
        Ok(mut comments) => {
            let base_url = state.storage_client.domain_cdn_url();

            let mut stack: Vec<&mut Comment> = comments.iter_mut().collect();
            while let Some(comment) = stack.pop() {
                if let Some(urls) = &mut comment.attachment_urls {
                    for url in urls.iter_mut() {
                        *url = format!("{}{}", base_url, url);
                    }
                }
                stack.extend(comment.replies.iter_mut());
            }
            (StatusCode::OK, Json(comments)).into_response()
        }
        Err(e) => {
            error!(
                "[CHAPTERS] Failed to get comments for chapter {}: {}",
                chapter_id, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to get comment for chapter"})),
            )
                .into_response()
        }
    }
}

// Helper function for post new comment handler
async fn new_comment_submission_handler(
    state: AppState,
    user: AuthenticatedUser,
    mut multipart: Multipart,
    entity_type: CommentEntityType,
    entity_id: i32,
) -> Response {
    let mut content_markdown = None;
    let mut parent_id: Option<i64> = None;
    let mut attachment_data: Vec<(Vec<u8>, String, String)> = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(field_name) = field.name() {
            match field_name {
                "content_markdown" => content_markdown = field.text().await.ok(),
                "parent_id" => {
                    if let Ok(text) = field.text().await {
                        parent_id = text.parse::<i64>().ok();
                    }
                }
                "images" => {
                    let file_name = field.file_name().unwrap_or("").to_string();
                    let content_type = field
                        .content_type()
                        .unwrap_or("application/octet-stream")
                        .to_string();
                    if let Ok(data) = field.bytes().await {
                        if data.len() > 5 * 1024 * 1024 {
                            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"message": "File size exceeds 5MB"}))).into_response();
                        }
                        attachment_data.push((data.to_vec(), file_name, content_type));
                    }
                }
                _ => (),
            }
        }
    }

    // Validation
    let content_markdown_str = content_markdown.unwrap_or_default();
    if content_markdown_str.is_empty() && attachment_data.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"message": "Comment must have content or an attachment."}))).into_response();
    }

    // Upload file if any
    let mut attachment_keys: Vec<String> = Vec::new();
    for (file_data, file_name, content_type) in attachment_data {
        let file_extension = std::path::Path::new(&file_name)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("");

        let unique_key =
            format!("comments/{}/{}.{}", user.id, Uuid::new_v4(), file_extension);

        if let Err(e) = state
            .storage_client
            .upload_image_file(file_data, &unique_key, &content_type)
            .await
        {
            error!("Failed to upload comment attachment: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to upload file attachment"})),
            )
                .into_response();
        }
        attachment_keys.push(unique_key);
    }

    // Create the new comment using the provided entity_type and entity_id
    let new_comment_id = match state
        .db_service
        .create_new_comment(
            user.id,
            entity_type,
            entity_id,
            &content_markdown_str,
            parent_id,
            &attachment_keys,
        )
        .await
    {
        Ok(id) => id,
        Err(e) => {
            error!(
                "Failed to create comment for entity type {:?} and ID {}: {}",
                entity_type, entity_id, e
            );
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
        Ok(Some(mut new_comment)) => {
            if let Some(urls) = &mut new_comment.attachment_urls {
                let base_url = state.storage_client.domain_cdn_url();
                for url in urls.iter_mut() {
                    *url = format!("{}{}", base_url, url);
                }
            }
            (StatusCode::OK, Json(new_comment)).into_response()
        },
        // Handle cases where the comment couldn't be fetched right after creation
        Ok(None) | Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Comment created but failed to retrieve its data"})),
        )
            .into_response(),
    }
}

// Post new comment to a specific series page
pub async fn post_series_comment_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(series_id): Path<i32>,
    multipart: Multipart,
) -> Response {
    println!(
        "->> {:<12} - record_series_view - series_id: {:?}",
        "HANDLER", series_id
    );

    new_comment_submission_handler(
        state,
        user,
        multipart,
        CommentEntityType::Series,
        series_id,
    )
    .await
}

// Post a new comment to a specific chapter
pub async fn post_chapter_comment_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(chapter_id): Path<i32>,
    multipart: Multipart,
) -> Response {
    println!(
        "->> {:<12} - record_series_view - series_id: {:?}",
        "HANDLER", chapter_id
    );

    new_comment_submission_handler(
        state,
        user,
        multipart,
        CommentEntityType::SeriesChapters,
        chapter_id,
    )
    .await
}

pub async fn upload_comment_attachments_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    mut multipart: Multipart,
) -> Response {
    if let Ok(Some(field)) = multipart.next_field().await {
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let file_name = field.file_name().unwrap_or("").to_string();

        let file_data = match field.bytes().await {
            Ok(bytes) => bytes.to_vec(),
            Err(e) => {
                return  (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"message": format!("Failed to read file: {}", e)})),
                    )
                    .into_response();
            }
        };

        const MAX_FILE_SIZE: usize = 5 * 1024 * 1024;
        if file_data.len() > MAX_FILE_SIZE {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"message": "File size cannot exceed 5MB"})),
            )
                .into_response();
        }

        let file_extension = std::path::Path::new(&file_name)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("");

        let unique_image_key =
            format!("comments/{}/{}.{}", user.id, Uuid::new_v4(), file_extension);

        match state
            .storage_client
            .upload_image_file(file_data, &unique_image_key, &content_type)
            .await
        {
            Ok(url) => {
                (StatusCode::OK, Json(serde_json::json!({"url": url}))).into_response()
            }
            Err(e) => {
                error!("Failed to upload comment attachment: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to upload file"})),
                )
                    .into_response()
            }
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"message": "No file found in the request."})),
        )
            .into_response()
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
        Ok(Some(_)) => {
            match state
                .db_service
                .get_comment_by_id(comment_id, Some(user.id))
                .await
            {
                Ok(Some(updated_comment)) => {
                    (
                        StatusCode::OK,
                        Json(updated_comment),
                    ).into_response()
                }
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"message": "Comment updated but failed to retrieve new data"})),
                )
                    .into_response(),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(
                serde_json::json!({"message": "Comment not found or permission denied"}),
            ),
        )
            .into_response(),
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

pub async fn vote_on_comment_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(comment_id): Path<i64>,
    Json(payload): Json<VotePayload>,
) -> Response {
    match state
        .db_service
        .vote_on_comment(comment_id, user.id, payload.vote_type)
        .await
    {
        Ok(response_data) => (StatusCode::OK, Json(response_data)).into_response(),
        Err(e) => {
            error!("Failed to vote on comment {}: {}", comment_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to vote on comment {}"})),
            )
                .into_response()
        }
    }
}

pub async fn get_all_categories_handler(State(state): State<AppState>) -> Response {
    match state.db_service.get_list_all_categories().await {
        Ok(categories) => (StatusCode::OK, Json(categories)).into_response(),
        Err(e) => {
            error!("Failed to get list of all categories: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Could not retrieve categories."})),
            )
                .into_response()
        }
    }
}

fn deserialize_i32_vec<'de, D>(deserializer: D) -> Result<Option<Vec<i32>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(s) if !s.is_empty() => {
            let result: Result<Vec<i32>, _> = s
                .split(',')
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .map(str::parse)
                .collect();

            match result {
                Ok(v) if !v.is_empty() => Ok(Some(v)),
                Ok(_) => Ok(None),
                Err(e) => Err(Error::custom(e)),
            }
        }
        _ => Ok(None),
    }
}

#[derive(Debug, Deserialize)]
pub struct BrowseParams {
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_pagesize")]
    page_size: u32,
    order_by: Option<String>,
    #[serde(default, deserialize_with = "deserialize_i32_vec")]
    include: Option<Vec<i32>>,
    #[serde(default, deserialize_with = "deserialize_i32_vec")]
    exclude: Option<Vec<i32>>,
}

pub async fn browse_series_handler(
    State(state): State<AppState>,
    Query(params): Query<BrowseParams>,
) -> Response {
    let order_by = match params.order_by.as_deref() {
        Some("new") => SeriesOrderBy::CreatedAt,
        Some("updated") => SeriesOrderBy::UpdatedAt,
        Some("views") => SeriesOrderBy::ViewsCount,
        Some("ratings") => SeriesOrderBy::Rating,
        _ => SeriesOrderBy::UpdatedAt,
    };

    let include_ids = params.include.as_deref().unwrap_or(&[]);
    let exclude_ids = params.exclude.as_deref().unwrap_or(&[]);

    match state
        .db_service
        .browse_series_paginated_with_filters(
            params.page,
            params.page_size,
            order_by,
            include_ids,
            exclude_ids,
        )
        .await
    {
        Ok(paginated_result) => (StatusCode::OK, Json(paginated_result)).into_response(),
        Err(e) => {
            error!("Failed to browse_series: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Could not retrieve series."})),
            ).into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct UserSearchParams {
    search: String,
}

pub async fn user_search_series_handler (
    State(state): State<AppState>,
    Query(params): Query<UserSearchParams>
) -> Response {
    match state.db_service.user_search_paginated_series(&params.search).await {
        Ok(series) => {
            (StatusCode::OK, Json(series)).into_response()
        },
        Err(e) => {
            error!("Failed to search series: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Could not search series."})),
            ).into_response()
        }
    }
}