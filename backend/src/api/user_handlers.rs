use crate::api::extractor::AuthenticatedUser;
use crate::builder::startup::AppState;
use crate::common::hashing::hash_password;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum_core::__private::tracing::error;
use axum_core::response::{IntoResponse, Response};
use axum_extra::extract::Multipart;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct UpdateProfilePayload {
    display_name: Option<String>,
    email: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePasswordPayload {
    new_password: String,
}

// fetch current logged-in user profile details
pub async fn get_user_profile_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Response {
    println!(
        "->> {:<12} - get_user_profile - series_id: {:?}",
        "HANDLER", user.id
    );

    match state.db_service.get_user_profile_details(user.id).await {
        Ok(Some(mut profile)) => {
            if let Some(key) = &profile.avatar_url {
                if !key.is_empty() {
                    profile.avatar_url = Some(format!(
                        "{}/{}",
                        state.storage_client.domain_cdn_url(),
                        key
                    ));
                }
            }
            (StatusCode::OK, Json(profile)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "message": "User profile not found"})),
        )
            .into_response(),
        Err(e) => {
            error!("DB error fetching user profiles: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"status": "error", "message": "Could not fetch user profiles"}))).into_response()
        }
    }
}

// Partially update user profile (display_name, email)
pub async fn update_user_profile_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<UpdateProfilePayload>,
) -> Response {
    // Validate email uniqueness if its being changed
    if let Some(ref email) = payload.email
        && let Ok(Some(existing_user)) =
            state.db_service.get_user_by_identifier(email).await
        && existing_user.id != user.id
    {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"message": "Email already in use"})),
        )
            .into_response();
    }

    // Call db to perform partial update
    match state
        .db_service
        .update_partial_user_profile(
            user.id,
            payload.display_name,
            payload.email,
        )
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "Profile update successfully"})),
        )
            .into_response(),
        Err(e) => {
            error!("DB error updating user profiles: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "Failed to update profile"})),
            )
                .into_response()
        }
    }
}

pub async fn update_user_password_setting_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<UpdatePasswordPayload>,
) -> Response {
    if payload.new_password.len() < 8 {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"message": "Password must be at least 8 characters long."}))).into_response();
    }

    let hashed_password = match hash_password(&payload.new_password) {
        Ok(hashed) => hashed,
        Err(_) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"message": "Failed to process password."})),
        )
            .into_response(),
    };

    match state
        .db_service
        .update_user_password(user.id, &hashed_password)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(
                serde_json::json!({"message": "Password updated successfully"}),
            ),
        )
            .into_response(),
        Err(e) => {
            error!("DB error updating password: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"message": "Could not update password."}))).into_response()
        }
    }
}

// Upload and update user avatar
pub async fn update_user_avatar_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    mut multipart: Multipart,
) -> Response {
    if let Ok(Some(field)) = multipart.next_field().await {
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let file_name = field.file_name().unwrap_or("unknown.jpg").to_string();

        let file_data = match field.bytes().await {
            Ok(bytes) => bytes.to_vec(),
            Err(e) => return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"message": format!("Failed to read file: {}", e)})),
            )
                .into_response(),
        };

        let file_extension = std::path::Path::new(&file_name)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("jpg");

        // Image avatar key (avatars/user_123_random-uuid.jpg)
        let unique_image_key = format!(
            "avatars/user_{}_{}.{}",
            user.id,
            Uuid::new_v4(),
            file_extension
        );

        // Upload to cloud storage
        return match state
            .storage_client
            .upload_image_file(file_data, &unique_image_key, &content_type)
            .await
        {
            Ok(key) => {
                match state.db_service.update_user_avatar(user.id, &key).await {
                    Ok(_) => {
                        // Construct the public URL
                        let public_url = format!("{}/{}", state.storage_client.domain_cdn_url(), key);

                        (StatusCode::OK, Json(serde_json::json!({"status": "success", "url": public_url}))).into_response()
                    },
                    Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"message": "Failed to save avatar URL."}))).into_response(),
                }
            }
            Err(e) => {
                error!("Error updating user avatar: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"message": "Failed to upload avatar."}))).into_response()
            }
        };
    }
    (
        StatusCode::BAD_REQUEST,
        Json(
            serde_json::json!({"message": "No avatar file found in request."}),
        ),
    )
        .into_response()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestChapterInfo {
    chapter_number: f32,
    title: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkStatusResponse {
    is_bookmarked: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkSeriesResponse {
    id: i32,
    title: String,
    cover_image_url: String,
    updated_at: DateTime<Utc>,
    latest_chapter: Option<LatestChapterInfo>,
}

// Add bookmark for the current user
pub async fn add_bookmark_series_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(series_id): Path<i32>,
) -> Response {
    match state.db_service.add_bookmarked_series(user.id, series_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "success", "message": "Add Bookmark"})),
        )
            .into_response(),
        Err(e) => {
            error!("DB error adding bookmark: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Could not add bookmark"})),
            )
                .into_response()
        }
    }
}

// Remove bookmark for the current user
pub async fn delete_bookmark_series_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(series_id): Path<i32>,
) -> Response {
    match state.db_service.delete_bookmarked_series(user.id, series_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "success", "message": "Remove Bookmark"})),
        )
            .into_response(),
        Err(e) => {
            error!("DB error fetching user bookmarks: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Could not remove bookmark"})),
            )
                .into_response()
        }
    }
}

// Check if a series is bookmarked by current user
pub async fn get_bookmark_status_current_user_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(series_id): Path<i32>,
) -> Response {
    match state
        .db_service
        .is_series_bookmarked(user.id, series_id)
        .await
    {
        Ok(is_bookmarked) => (
            StatusCode::OK,
            Json(BookmarkStatusResponse { is_bookmarked }),
        )
            .into_response(),
        Err(e) => {
            error!("DB error fetching user bookmarks: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Could not fetch bookmarks"})),
            )
                .into_response()
        }
    }
}

// Fetch all bookmarked series for user
pub async fn get_user_bookmark_library_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Response {
    match state
        .db_service
        .get_bookmarked_series_for_user(user.id)
        .await
    {
        Ok(bookmarked_series_list) => {
            let response_list = bookmarked_series_list
                .into_iter()
                .map(|series| {
                    let latest_chapter_info = series
                        .last_chapter_found_in_storage
                        .map(|chapter_num| LatestChapterInfo {
                            chapter_number: chapter_num,
                            title: series.chapter_title,
                        });

                    BookmarkSeriesResponse {
                        id: series.id,
                        title: series.title,
                        cover_image_url: series.cover_image_url,
                        updated_at: series.updated_at,
                        latest_chapter: latest_chapter_info,
                    }
                })
                .collect::<Vec<_>>();

            (StatusCode::OK, Json(response_list)).into_response()
        }
        Err(e) => {
            error!("DB error fetching user bookmarks: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"status": "error", "message": "Could not fetch user bookmarks"}))
            )
                .into_response()
        }
    }
}
