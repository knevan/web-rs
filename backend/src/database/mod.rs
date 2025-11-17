use std::fmt;

use anyhow::{Context, Result as AnyhowResult};
use chrono::{DateTime, Utc};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};
use url::Url;

pub mod auth;
pub mod chapters;
pub mod comments;
pub mod series;
pub mod series_user_actions;
pub mod storage;
pub mod users;

// Type alias for database connection pool
pub type DbPool = PgPool;

// Database operations with connection pool
#[derive(Clone)]
pub struct DatabaseService {
    pool: DbPool,
}

impl DatabaseService {
    pub fn new(pool: DbPool) -> Self {
        DatabaseService { pool }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "series_status", rename_all = "PascalCase")]
pub enum SeriesStatus {
    Pending,
    Processing,
    Available,
    Ongoing,
    Completed,
    Hiatus,
    Discontinued,
    Error,
    #[sqlx(rename = "Pending Deletion")]
    PendingDeletion,
    Deleting,
    #[sqlx(rename = "Deletion Failed")]
    DeletionFailed,
}

impl fmt::Display for SeriesStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_str = match self {
            SeriesStatus::Pending => "Pending",
            SeriesStatus::Processing => "Processing",
            SeriesStatus::Available => "Available",
            SeriesStatus::Ongoing => "Ongoing",
            SeriesStatus::Completed => "Completed",
            SeriesStatus::Hiatus => "Hiatus",
            SeriesStatus::Discontinued => "Discontinued",
            SeriesStatus::Error => "Error",
            SeriesStatus::PendingDeletion => "PendingDeletion",
            SeriesStatus::Deleting => "Deleting",
            SeriesStatus::DeletionFailed => "DeletionFailed",
        };
        write!(f, "{}", status_str)
    }
}

// Struct represents a series stored in the database.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Series {
    pub id: i32,
    pub title: String,
    pub original_title: Option<String>,
    pub description: String,
    pub cover_image_url: String,
    pub current_source_url: String,
    pub source_website_host: String,
    pub views_count: i32,
    pub bookmarks_count: i32,
    pub total_rating_score: i64,
    pub total_ratings_count: i32,
    pub last_chapter_found_in_storage: Option<f32>, // support 10.0, 10.5
    pub processing_status: SeriesStatus,
    pub check_interval_minutes: i32,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub next_checked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "series_status", rename_all = "PascalCase")]
pub enum ChapterStatus {
    Processing,
    Available,
    NoImagesFound,
    Error,
}

/// Struct represent chapter
#[derive(Debug, FromRow, Serialize)]
pub struct SeriesChapter {
    pub id: i32,
    pub series_id: i32,
    pub chapter_number: f32,
    pub title: Option<String>,
    pub status: ChapterStatus,
    pub source_url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct NewSeriesData<'a> {
    pub title: &'a str,
    pub original_title: Option<&'a str>,
    pub authors: Option<&'a Vec<String>>,
    pub category_ids: Option<&'a Vec<i32>>,
    pub description: &'a str,
    pub cover_image_url: &'a str,
    pub source_url: &'a str,
    pub check_interval_minutes: i32,
}

#[derive(Debug, Default)]
pub struct UpdateSeriesData<'a> {
    pub title: Option<&'a str>,
    pub original_title: Option<&'a str>,
    pub authors: Option<&'a Vec<String>>,
    pub description: Option<&'a str>,
    pub cover_image_url: Option<&'a str>,
    pub source_url: Option<&'a str>,
    pub check_interval_minutes: Option<i32>,
    pub category_ids: Option<&'a [i32]>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct SeriesWithAuthors {
    pub id: i32,
    pub title: String,
    pub original_title: Option<String>,
    pub description: String,
    pub cover_image_url: String,
    pub current_source_url: String,
    pub processing_status: SeriesStatus,
    pub updated_at: DateTime<Utc>,
    #[sqlx(json)]
    pub authors: serde_json::Value,
}

#[derive(Debug)]
pub struct SeriesDeletionImagekeys {
    pub cover_image_url: Option<String>,
    pub chapter_image_urls: Vec<String>,
}

impl SeriesDeletionImagekeys {
    // Helper function to get all urls in a single iteration
    pub fn all_urls(&self) -> impl Iterator<Item = &str> {
        // Combine cover image and chapter image urls into a single iterator
        self.cover_image_url
            .as_deref()
            .into_iter()
            .chain(self.chapter_image_urls.iter().map(String::as_str))
    }
}

// Order by field for fetching series list.
#[derive(Debug, Clone)]
pub enum SeriesOrderBy {
    CreatedAt,
    UpdatedAt,
    ViewsCount,
    Rating,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct CategoryTag {
    pub id: i32,
    pub name: String,
}

// Pagination parameters for fetching series list.
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total_items: i64,
}

// User Search Paginated Series Struct
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserSearchPaginatedSeries {
    pub id: i32,
    pub title: String,
    pub original_title: Option<String>,
    pub cover_image_url: String,
    pub last_chapter_found_in_storage: Option<f32>,
    pub updated_at: DateTime<Utc>,
    #[sqlx(json)]
    pub authors: serde_json::Value,
}

// Latest Release Series Struct
#[derive(Debug, Serialize, FromRow)]
pub struct LatestReleaseSeries {
    pub id: i32,
    pub title: String,
    pub original_title: Option<String>,
    #[sqlx(json)]
    pub authors: serde_json::Value,
    pub cover_image_url: String,
    pub description: String,
    pub last_chapter_found_in_storage: Option<f32>,
    pub updated_at: DateTime<Utc>,
    pub chapter_title: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct BrowseSeriesSearchResult {
    pub id: i32,
    pub title: String,
    pub original_title: Option<String>,
    pub description: String,
    pub cover_image_url: String,
    pub last_chapter_found_in_storage: Option<f32>,
    pub updated_at: DateTime<Utc>,
    #[sqlx(json)]
    pub authors: serde_json::Value,
    #[sqlx(json)]
    pub categories: serde_json::Value,
}

// Most viewed series data for the public API.
#[derive(Debug, FromRow, Serialize)]
pub struct MostViewedSeries {
    pub id: i32,
    pub title: String,
    pub cover_image_url: String,
    pub view_count: Option<i64>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct BookmarkedSeries {
    pub id: i32,
    pub title: String,
    pub cover_image_url: String,
    pub last_chapter_found_in_storage: Option<f32>,
    pub updated_at: DateTime<Utc>,
    pub chapter_title: Option<String>,
}

/// Strcuct represents a user record fetched from the database
#[derive(Debug, FromRow)]
pub struct Users {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role_id: i32,
}

#[derive(Debug, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfileDetails {
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct UserWithRole {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub role_name: String,
}

// Comment struct
#[derive(Debug, FromRow, Serialize, Clone)]
pub struct Comment {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub content_html: String,
    pub content_markdown: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user: CommentUser,
    pub upvotes: i64,
    pub downvotes: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_user_vote: Option<i16>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub replies: Vec<Comment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_urls: Option<Vec<String>>,
}

// Helper struct to map the flat comment result
#[derive(Debug, FromRow)]
struct CommentFlatRow {
    id: i64,
    parent_id: Option<i64>,
    content_html: String,
    content_markdown: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    user_id: i32,
    user_username: String,
    user_avatar_url: Option<String>,
    upvotes: i64,
    downvotes: i64,
    current_user_vote: Option<i16>,
    attachment_urls: Option<serde_json::Value>,
}

impl From<CommentFlatRow> for Comment {
    fn from(row: CommentFlatRow) -> Self {
        Comment {
            id: row.id,
            parent_id: row.parent_id,
            content_html: row.content_html,
            content_markdown: row.content_markdown,
            created_at: row.created_at,
            updated_at: row.updated_at,
            user: CommentUser {
                id: row.user_id,
                username: row.user_username,
                avatar_url: row.user_avatar_url,
            },
            upvotes: row.upvotes,
            downvotes: row.downvotes,
            current_user_vote: row.current_user_vote,
            replies: Vec::new(),
            attachment_urls: row
                .attachment_urls
                .and_then(|v| serde_json::from_value(v).ok()),
        }
    }
}

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct CommentUser {
    pub id: i32,
    pub username: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "comments_entity", rename_all = "snake_case")]
pub enum CommentEntityType {
    Series,
    SeriesChapters,
}

#[derive(Debug, Deserialize)]
pub struct NewCommentPayload {
    pub content_markdown: String,
    pub parent_id: Option<i64>,
    pub attachments: Option<Vec<String>>,
}

#[derive(FromRow, Serialize, Debug)]
pub struct UpdateCommentResponse {
    pub id: i64,
    pub content_user_markdown: String,
    pub content_html: String,
    pub updated_at: DateTime<Utc>,
}

// Payload for voting on a comment.
#[derive(Debug, Deserialize)]
pub struct VotePayload {
    // 1 for upvote, -1 for downvote, 0 to remove vote
    pub vote_type: i16,
}

#[derive(Debug, Serialize)]
pub struct CommentVoteResponse {
    pub new_upvotes: i64,
    pub new_downvotes: i64,
    pub current_user_vote: Option<i16>,
}

// A helper function to extract a hostname from an optional URL string.
// This is created to avoid code duplication, following the DRY principle.
fn get_host_from_url(url_option: Option<&str>) -> Option<String> {
    url_option.and_then(|url_str| {
        Url::parse(url_str)
            .ok()
            .and_then(|url| url.host_str().map(String::from))
    })
}
