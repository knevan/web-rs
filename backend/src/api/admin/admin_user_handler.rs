use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum_core::__private::tracing::error;
use axum_core::response::{IntoResponse, Response};
use serde_json::Value;

use crate::api::admin::{
    AdminError, AdminResult, AdminUpdateUserPayload, PaginatedResponse, PaginationParams,
};
use crate::api::extractor::AdminOrHigherUser;
use crate::builder::startup::AppState;
use crate::database::UserWithRole;

// Fetch all users with pagination
pub async fn get_all_paginated_users_handler(
    admin: AdminOrHigherUser,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
) -> Response {
    println!(
        "->> {:<12} - get_all_users_handler - user: {}",
        "HANDLER", admin.0.username
    );

    match state
        .db_service
        .get_admin_paginated_user(
            pagination.page,
            pagination.page_size,
            pagination.search.as_deref(),
        )
        .await
    {
        Ok(paginated_result) => {
            let response_user_data = PaginatedResponse {
                items: paginated_result.items,
                total_items: paginated_result.total_items,
            };

            (StatusCode::OK, Json(response_user_data)).into_response()
        }
        Err(e) => {
            error!("Failed to get paginated users: {:#?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Could not retrieve users"})),
            )
                .into_response()
        }
    }
}

/// Partial update user metadata
pub async fn update_user_metadata_handler(
    admin: AdminOrHigherUser,
    State(state): State<AppState>,
    Path(user_id): Path<i32>,
    Json(payload): Json<AdminUpdateUserPayload>,
) -> AdminResult<Json<UserWithRole>> {
    println!(
        "->> {:<12} - update_user_handler - user: {}",
        "HANDLER", admin.0.username
    );

    if admin.0.id == user_id {
        return Err(AdminError::Forbidden(
            "You cannot update your own account".to_string(),
        ));
    }

    let updated_user = state
        .db_service
        .admin_update_user(
            user_id,
            payload.username.as_deref(),
            payload.email.as_deref(),
            payload.role_id,
            payload.is_active,
            admin.0.role,
        )
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("User with id {} not found", user_id)))?;

    Ok(Json(updated_user))
}

pub async fn delete_user_handler(
    State(state): State<AppState>,
    admin: AdminOrHigherUser,
    Path(user_id): Path<i32>,
) -> AdminResult<Json<Value>> {
    println!(
        "->> {:<12} - delete_user_handler - user: {}",
        "HANDLER", admin.0.username
    );

    let deleted_user = state.db_service.admin_delete_user(user_id).await?;

    if deleted_user == 0 {
        return Err(AdminError::NotFound(format!(
            "User with id {} not found",
            user_id
        )));
    }

    Ok(Json(
        serde_json::json!({"status": "success", "message": "User deleted successfully"}),
    ))
}
