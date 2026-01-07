use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum_core::__private::tracing::error;
use axum_core::response::{IntoResponse, Response};

use crate::api::extractor::AuthenticatedUser;
use crate::builder::startup::AppState;
use crate::database::{CreateChapterReportRequest, ReportTarget};

pub async fn report_chapter_handler(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(chapter_id): Path<i32>,
    Json(payload): Json<CreateChapterReportRequest>,
) -> Response {
    let reason = payload.reason.into();

    match state
        .db_service
        .user_submit_report(user.id, ReportTarget::Chapter(chapter_id), reason)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "message": "Report submitted successfully. Thank you for helping us."
            })),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to submit report for chapter {}: {}", chapter_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "message": "Failed to submit report."
                })),
            )
                .into_response()
        }
    }
}
