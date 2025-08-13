use anyhow::{Context, Result as AnyhowResult};
use chrono::{DateTime, Utc};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use url::Url;

pub mod auth;
pub mod chapters;
pub mod db;
pub mod series;
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

// Struct represents a manga series stored in the database.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Series {
    pub id: i32,
    pub title: String,
    pub original_title: String,
    pub description: String,
    pub cover_image_url: String,
    pub current_source_url: String,
    pub source_website_host: String,
    pub views_count: i32,
    pub bookmarks_count: i32,
    pub last_chapter_found_in_storage: Option<f32>, // support 10.0, 10.5
    pub processing_status: String, // "pending", "monitoring", "error", "completed"
    pub check_interval_minutes: i32,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub next_checked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Struct represent chapter
#[derive(Debug, FromRow, Serialize)]
pub struct SeriesChapter {
    pub id: i32,
    pub series_id: i32,
    pub chapter_number: f32,
    pub title: Option<String>,
    pub source_url: String,
    pub created_at: DateTime<Utc>,
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
}

#[derive(Debug, FromRow, Serialize)]
pub struct SeriesWithAuthors {
    pub id: i32,
    pub title: String,
    pub original_title: String,
    pub description: String,
    pub cover_image_url: String,
    pub current_source_url: String,
    pub processing_status: String,
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

// Pagination parameters for fetching series list.
#[derive(Debug)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total_items: i64,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct CategoryTag {
    pub id: i32,
    pub name: String,
}

// Public series data for the public API.
#[derive(Debug, FromRow, Serialize)]
pub struct PublicSeries {
    pub id: i32,
    pub title: String,
    pub cover_image_url: String,
    // Use serde_json::Value for authors to match SeriesWithAuthors, then map it in the handler
    #[sqlx(json)]
    pub authors: serde_json::Value,
}

// Most viewed series data for the public API.
#[derive(Debug, FromRow, Serialize)]
pub struct MostViewedSeries {
    pub id: i32,
    pub title: String,
    pub cover_image_url: String,
    pub view_count: Option<i64>,
}

// Order by field for fetching series list.
#[derive(Debug, Clone)]
pub enum SeriesOrderBy {
    CreatedAt,
    UpdatedAt,
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
