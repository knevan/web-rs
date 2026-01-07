use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum_core::__private::tracing::{error, info};
use axum_core::response::{IntoResponse, Response};
use serde_json::json;

use crate::api::admin::ReportPaginationParams;
use crate::api::extractor::{AdminOrHigherUser, ModeratorOrHigherUser};
use crate::builder::startup::AppState;
use crate::database::{DatabaseService, DeleteCommentResult, UpdateCommentResponse};

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

pub async fn list_reports_handler(
    admin: AdminOrHigherUser,
    State(state): State<AppState>,
    Query(params): Query<ReportPaginationParams>,
) -> Response {
    info!(
        "->> {:<12} - list_reports_handler - user: {}",
        "HANDLER", admin.0.username
    );

    match state
        .db_service
        .get_admin_paginated_pending_reports(
            params.page,
            params.page_size,
            params.search.as_deref(),
        )
        .await
    {
        Ok(paginated_result) => (
            StatusCode::OK,
            Json(json!({"status": "success", "data": paginated_result})),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to fetch reports: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error", "message": "Failed to retrieve reports."})),
            )
                .into_response()
        }
    }
}

pub async fn resolve_report_handler(
    admin: AdminOrHigherUser,
    State(state): State<AppState>,
    Path(report_id): Path<i32>,
) -> Response {
    info!(
        "->> {:<12} - resolve_report_handler - user: {}, report_id: {}",
        "HANDLER", admin.0.username, report_id
    );

    match state.db_service.admin_resolve_reports(report_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"status": "success", "message": format!("Report #{} resolved and cleared.", report_id)})),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to resolve report {}: {}", report_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error", "message": "Failed to resolve report."})),
            )
                .into_response()
        }
    }
}
