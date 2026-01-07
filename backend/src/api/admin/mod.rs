use axum::Json;
use axum_core::__private::tracing::log::error;
use axum_core::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::common::error::AuthError;
use crate::database::ReportReason;

pub mod admin_actions_handler;
pub mod admin_routes;
pub mod admin_series_handlers;
pub mod admin_user_handler;

#[derive(Serialize)]
struct AdminErrorResponse {
    error: String,
}

pub enum AdminError {
    Auth(AuthError),
    Conflict(String),
    NotFound(String),
    Forbidden(String),
    DatabaseError(anyhow::Error),
}

impl From<anyhow::Error> for AdminError {
    fn from(e: anyhow::Error) -> Self {
        let e_str = e.to_string();
        if e.to_string().contains("Username or Email already exists") {
            AdminError::Conflict("Username or Email already exists".to_string())
        } else if e_str.starts_with("FORBIDDEN:") {
            AdminError::Forbidden(e_str.replace("FORBIDDEN: ", ""))
        } else {
            AdminError::DatabaseError(e)
        }
    }
}

impl From<AuthError> for AdminError {
    fn from(e: AuthError) -> Self {
        AdminError::Auth(e)
    }
}

impl IntoResponse for AdminError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AdminError::DatabaseError(e) => {
                error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal server error occurred".to_string(),
                )
            }
            AdminError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AdminError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AdminError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AdminError::Auth(e) => {
                return e.into_response();
            }
        };

        let body = Json(AdminErrorResponse { error: message });
        (status, body).into_response()
    }
}

pub type AdminResult<T> = Result<T, AdminError>;

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

#[derive(Serialize)]
pub struct UploadCoverImageResponse {
    status: String,
    url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
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
pub struct PaginatedResponse<T: Serialize> {
    items: Vec<T>,
    total_items: i64,
}

#[derive(Deserialize)]
pub struct RepairChapterRequest {
    pub chapter_number: f32,
    pub new_chapter_url: String,
}

#[derive(Deserialize)]
pub struct CreateCategoryTagRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct AdminUpdateUserPayload {
    username: Option<String>,
    email: Option<String>,
    role_id: Option<i32>,
    is_active: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportPaginationParams {
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_page_size")]
    page_size: u32,
    #[serde(default)]
    search: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportView {
    pub id: i32,
    pub reporter_username: String,
    pub reporter_id: i32,
    pub created_at: DateTime<Utc>,
    pub reason: ReportReason,
    pub chapter_id: Option<i32>,
    pub chapter_number: Option<f32>,
    pub chapter_series_title: Option<String>,
    pub comment_id: Option<i64>,
    pub comment_preview: Option<String>,
}
