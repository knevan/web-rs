use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum_core::__private::tracing::error;
use axum_core::response::{IntoResponse, Response};
use axum_extra::extract::Multipart;
use serde::Deserialize;
use uuid::Uuid;

use crate::api::extractor::{AuthenticatedUser, OptionalAuthenticatedUser};
use crate::api::public::user_handlers::extract_field_data;
use crate::builder::startup::AppState;
use crate::database::{Comment, CommentEntityType, CommentSort, DeleteCommentResult, VotePayload};

/// Helper function to recursively prepend the base CDN URL to all comment attachment URLs.
/// This modifies the comments in place using an iterative stack-based approach.
fn hydrate_attachments_url(comments: &mut [Comment], base_url: &str) {
    // Collect mutable references to the top-level comments
    let mut stack: Vec<&mut Comment> = comments.iter_mut().collect();

    // Process comments iteratively to avoid deep recursion
    while let Some(comment) = stack.pop() {
        // Prepend base_url to attachment_urls if they exist
        if let Some(urls) = &mut comment.attachment_urls {
            for url in urls.iter_mut() {
                *url = format!("{}/{}", base_url, url);
            }
        }

        stack.extend(comment.replies.iter_mut());
    }
}

#[derive(Deserialize)]
pub struct CommentParams {
    #[serde(default)]
    sort: CommentSort,
    thread_id: Option<i64>,
}

// Fetch series comments
pub async fn get_series_comment_handler(
    State(state): State<AppState>,
    Path(series_id): Path<i32>,
    user: OptionalAuthenticatedUser,
    Query(params): Query<CommentParams>,
) -> Response {
    let user_id = user.0.map(|u| u.id);
    match state
        .db_service
        .get_comments(
            CommentEntityType::Series,
            series_id,
            user_id,
            params.sort,
            params.thread_id,
        )
        .await
    {
        Ok(mut comments) => {
            let base_url = state.storage_client.domain_cdn_url();

            hydrate_attachments_url(&mut comments, base_url);

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
    Query(params): Query<CommentParams>,
) -> Response {
    let user_id = user.0.map(|u| u.id);
    match state
        .db_service
        .get_comments(
            CommentEntityType::SeriesChapters,
            chapter_id,
            user_id,
            params.sort,
            params.thread_id,
        )
        .await
    {
        Ok(mut comments) => {
            let base_url = state.storage_client.domain_cdn_url();

            hydrate_attachments_url(&mut comments, base_url);

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

// New comment send handler w/o attachments
pub async fn new_comment_submission_handler(
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
                            return (
                                StatusCode::BAD_REQUEST,
                                Json(serde_json::json!({"message": "File size exceeds 5MB"})),
                            )
                                .into_response();
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
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"message": "Comment must have content or an attachment."})),
        )
            .into_response();
    }

    // Upload file if any
    let mut attachment_keys: Vec<String> = Vec::new();
    for (file_data, file_name, content_type) in attachment_data {
        let file_extension = std::path::Path::new(&file_name)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("");

        let unique_key = format!("comments/{}/{}.{}", user.id, Uuid::new_v4(), file_extension);

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
            let base_url = state.storage_client.domain_cdn_url();

            hydrate_attachments_url(std::slice::from_mut(&mut new_comment), base_url);

            (StatusCode::OK, Json(new_comment)).into_response()
        }
        // Handle cases where the comment couldn't be fetched right after creation
        Ok(None) => {
            error!(
                "Comment created id {} but not not found immediately early",
                new_comment_id
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Comment create but not found early"})),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to retrive comment data {:#?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({"error": format!("Comment created but failed to retrieve its data {:#?}", e)}),
                ),
            )
                .into_response()
        }
    }
}

// Create/Post new comment to a specific series page
pub async fn create_series_comment_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(series_id): Path<i32>,
    multipart: Multipart,
) -> Response {
    println!(
        "->> {:<12} - record_series_view - series_id: {:?}",
        "HANDLER", series_id
    );

    new_comment_submission_handler(state, user, multipart, CommentEntityType::Series, series_id)
        .await
}

// Create/Post a new comment to a specific chapter page
pub async fn create_chapter_comment_handler(
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

// Upload comment with attachments
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

        let file_data = match extract_field_data(field).await {
            Ok(data) => data,
            Err(response) => return response,
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
            Ok(url) => (StatusCode::OK, Json(serde_json::json!({"url": url}))).into_response(),
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

// Delete comment
pub async fn delete_comment_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(comment_id): Path<i64>,
) -> Response {
    match state.db_service.delete_comment(comment_id, user.id).await {
        Ok(DeleteCommentResult::SoftDeleted(updated_comment, attachment_object_key)) => {
            // Database transaction was successful.
            // Run this after the DB commit.
            if !attachment_object_key.is_empty()
                && let Err(e) = state
                    .storage_client
                    .delete_image_objects(&attachment_object_key)
                    .await
            {
                error!(
                    "Failed to delete storage objects for comment {}: {}. Keys: {:?}",
                    comment_id, e, attachment_object_key
                );
            }
            (StatusCode::OK, Json(updated_comment)).into_response()
        }
        Ok(DeleteCommentResult::InsufficientPermissions) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "status": "error",
                "message": "You do not have permission to delete this user's comment"
            })),
        )
            .into_response(),
        Ok(DeleteCommentResult::HardDeleted(attachment_object_key)) => {
            if !attachment_object_key.is_empty()
                && let Err(e) = state
                    .storage_client
                    .delete_image_objects(&attachment_object_key)
                    .await
            {
                error!(
                    "Failed to delete storage objects for comment {}: {}. Keys: {:?}",
                    comment_id, e, attachment_object_key
                );
            }
            (
                StatusCode::NO_CONTENT,
                Json(serde_json::json!({"message": "Comment not found or permission denied"})),
            )
                .into_response()
        }
        Ok(DeleteCommentResult::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"message": "Comment not found or permission denied"})),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to delete comment with id: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to delete comment"})),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateCommentPayload {
    pub content_markdown: String,
}

// Update (Edit) existing comment
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
        Ok(Some(updated_data)) => (StatusCode::OK, Json(updated_data)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"message": "Comment not found or permission denied"})),
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

// Vote on comment
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
