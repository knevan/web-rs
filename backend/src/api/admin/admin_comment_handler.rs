use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_core::__private::tracing::{error, info};
use axum_core::response::{IntoResponse, Response};
use serde_json::json;

use crate::api::extractor::ModeratorOrHigherUser;
use crate::builder::startup::AppState;
use crate::database::{DeleteCommentResult, UpdateCommentResponse};

pub async fn admin_delete_comment_handler(
    auth: ModeratorOrHigherUser,
    State(state): State<AppState>,
    Path(comment_id): Path<i64>,
) -> Response {
    println!(
        "->> {:<12} - admin_delete_comment - mod: {}, comment_id: {}",
        "HANDLER", auth.0.username, comment_id
    );

    let user_info = format!("{} (ID: {})", auth.0.username, auth.0.id);
    let requestor_role_id = auth.0.role as i32;

    match state
        .db_service
        .admin_delete_comment(comment_id, requestor_role_id)
        .await
    {
        Ok(result) => {
            let mut soft_deleted_data: Option<UpdateCommentResponse> = None;
            let mut delete_type = "hard_delete";

            let (status_msg, files_to_delete) = match result {
                DeleteCommentResult::NotFound => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(json!({
                            "status": "error",
                            "message": format!("Comment with id {} not found", comment_id)
                        })),
                    )
                        .into_response();
                }
                DeleteCommentResult::InsufficientPermissions => {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(json!({
                            "status": "error",
                            "message": "You do not have permission to delete this user's comment"
                        })),
                    )
                        .into_response();
                }
                DeleteCommentResult::SoftDeleted(updated_comment, keys) => {
                    soft_deleted_data = Some(updated_comment);
                    delete_type = "soft_delete";
                    ("Comment soft-deleted (replies exist)", keys)
                }
                DeleteCommentResult::HardDeleted(keys) => ("Comment hard-deleted", keys),
            };

            let files_count = files_to_delete.len();

            if !files_to_delete.is_empty() {
                let storage = state.storage_client.clone();
                let mod_name = auth.0.username;

                tokio::spawn(async move {
                    match storage.delete_image_objects(&files_to_delete).await {
                        Ok(_) => info!(
                            "Background: Deleted {} files for comment {} by {}",
                            files_count, comment_id, mod_name
                        ),
                        Err(e) => error!(
                            "Background: Failed to delete files for comment {}: {:?}",
                            comment_id, e
                        ),
                    }
                });
            }
            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "message": status_msg,
                    "action_type": delete_type,
                    "comment": soft_deleted_data,
                    "deleted_files_count": files_count,
                    "moderated_by": user_info
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to admin delete comment: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": "Internal server error during comment deletion"
                })),
            )
                .into_response()
        }
    }
}
