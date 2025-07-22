use anyhow::{Context, Result as AnyhowResult, anyhow};
use chrono::{DateTime, Utc};
use rand::prelude::*;
use sqlx::{FromRow, PgPool};
use url::Url;

// Type alias for db connection pool
pub type DbPool = PgPool;

/// Struct represents a manga series stored in the database.
/// The derive macro is now `sqlx::FromRow`.
#[derive(Debug, Clone, FromRow)]
pub struct MangaSeries {
    pub id: i32,
    pub title: String,
    pub original_title: String,
    pub description: String,
    pub cover_image_url: String,
    pub current_source_url: String,
    pub source_website_host: String,
    pub views_count: i32,
    pub bookmarks_count: i32,
    pub last_chapter_found_in_storage: Option<f32>, // e.g., 10.0, 10.5
    pub processing_status: String, // e.g., "pending", "monitoring", "error", "completed"
    pub check_interval_minutes: i32,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub next_checked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Struct represent chapter
#[derive(Debug, FromRow)]
pub struct Chapter {
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
pub struct NewMangaSeriesData<'a> {
    pub title: &'a str,
    pub original_title: Option<&'a str>,
    pub authors: Option<&'a Vec<String>>,
    pub description: &'a str,
    pub cover_image_url: &'a str,
    pub source_url: &'a str,
    pub check_interval_minutes: i32,
}

#[derive(Debug, Default)]
pub struct UpdateMangaSeriesData<'a> {
    pub title: Option<&'a str>,
    pub original_title: Option<&'a str>,
    pub authors: Option<&'a Vec<String>>,
    pub description: Option<&'a str>,
    pub cover_image_url: Option<&'a str>,
    pub source_url: Option<&'a str>,
    pub check_interval_minutes: Option<i32>,
}

#[derive(Debug, FromRow)]
pub struct SeriesWithAuthors {
    pub id: i32,
    pub title: String,
    pub original_title: String,
    pub description: String,
    pub cover_image_url: String,
    pub current_source_url: String,
    pub updated_at: DateTime<Utc>,
    #[sqlx(json)]
    pub authors: serde_json::Value,
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

// Database operations with connection pool
#[derive(Clone)]
pub struct DatabaseService {
    pool: DbPool,
}

#[derive(Debug)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total_items: i64,
}

/// Macros `sqlx::query!`
/// For DML operations (INSERT, UPDATE, DELETE) or SELECTs,
/// where you're manually processing generic `sqlx::Row`s (anonymous struct).
///
/// Macros `sqlx::query_as!`
/// For mapping SELECT results directly to a defined rust struct (`#[derive(FromRow)]`),
/// recommended for structured data retrieval.
///
/// Macros `sqlx::query_scalar!`
/// For queries returning a single value (one row, one column).
/// Highly efficient for this purpose.
///
/// Use .execute() when you want to run a command and don't need any row data back.
/// UPDATE, DELETE, INSERT (without a RETURNING clause), or CREATE TABLE. It's fire-and-forget.
///
/// Use .fetch_one() when you are certain the query will return EXACTLY one row
/// It will error if it gets zero or more than one row. Useful for fetching by a primary key.
/// SELECT ... WHERE id = ? or INSERT ... RETURNING id. (Your logic requires a single, unique record to exist.)
///
/// Use .fetch_optional() when a record may or may not exist, the query could return one row or nothing.
/// It will be Some(data) if a row is found, None if no rows are found, Error if more than one row.
/// Use for checking if a user exists with SELECT ... WHERE email = ?.
impl DatabaseService {
    pub fn new(pool: DbPool) -> Self {
        DatabaseService { pool }
    }

    pub async fn add_new_manga_series(
        &self,
        data: &NewMangaSeriesData<'_>,
    ) -> AnyhowResult<i32> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to begin transaction")?;

        let host = get_host_from_url(Some(data.source_url));

        let new_manga_id = sqlx::query_scalar!(
            r#"INSERT INTO manga_series
            (title, original_title, description, cover_image_url, current_source_url, source_website_host, check_interval_minutes)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id"#,
            data.title,
            data.original_title,
            data.description,
            data.cover_image_url,
            data.source_url,
            host,
            data.check_interval_minutes,
        )
            .fetch_one(&mut *tx)
            .await
            .context("Failed to add manga series with sqlx")?;

        if let Some(author_names) = data.authors {
            for name in author_names {
                let author_id = sqlx::query_scalar!(
                    r#"
                    WITH ins AS(
                        INSERT INTO authors (name)
                        VALUES ($1)
                        ON CONFLICT (name) DO NOTHING
                        RETURNING id
                    )
                    SELECT id FROM ins
                    UNION ALL
                    SELECT id FROM authors WHERE name = $1
                    LIMIT 1
                    "#,
                    name
                )
                .fetch_one(&mut *tx)
                .await
                .context("Failed to find or create author with sqlx")?;

                sqlx::query!(
                    "INSERT INTO manga_authors (manga_id, author_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                    new_manga_id,
                    author_id
                ).execute(&mut *tx).await.context(format!("Failed to link author {} to manga", name))?;
            }
        }
        tx.commit().await.context("Failed to commit transaction")?;

        Ok(new_manga_id)
    }

    pub async fn update_manga_series_metadata(
        &self,
        series_id: i32,
        data: &UpdateMangaSeriesData<'_>,
    ) -> AnyhowResult<u64> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to begin transaction")?;

        let host = get_host_from_url(data.source_url);

        let result = sqlx::query!(
            "UPDATE manga_series
            SET
                title = COALESCE($1, title),
                original_title = COALESCE($2, original_title),
                description = COALESCE($3, description),
                cover_image_url = COALESCE($4, cover_image_url),
                current_source_url = COALESCE($5, current_source_url),
                source_website_host = COALESCE($6, source_website_host),
                check_interval_minutes = COALESCE($7, check_interval_minutes),
            updated_at = NOW()
            WHERE id = $8",
            data.title,
            data.original_title,
            data.description,
            data.cover_image_url,
            data.source_url,
            host,
            data.check_interval_minutes,
            series_id
        )
        .execute(&mut *tx)
        .await
        .context("Failed to update manga series with sqlx")?;

        if let Some(author_names) = data.authors {
            sqlx::query!(
                "DELETE FROM manga_authors WHERE manga_id = $1",
                series_id
            )
            .execute(&mut *tx)
            .await
            .context("Failed to delete existing authors for manga")?;

            for name in author_names {
                let author_id = sqlx::query_scalar!(
                    r#"
                    WITH ins AS (
                        INSERT INTO authors (name) VALUES ($1)
                        ON CONFLICT (name) DO NOTHING
                        RETURNING id
                    )
                    SELECT id FROM ins
                    UNION ALL
                    SELECT id FROM authors WHERE name = $1
                    LIMIT 1
                    "#,
                    name
                )
                .fetch_one(&mut *tx)
                .await
                .context(format!(
                    "Failed to find or create author: {}",
                    name
                ))?;

                sqlx::query!(
                    "INSERT INTO manga_authors (manga_id, author_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                    series_id,
                    author_id
                )
                .execute(&mut *tx)
                   .await
                   .context(format!("Failed to link author {} to manga", name))?;
            }
        }
        tx.commit().await.context("Failed to commit transaction")?;

        Ok(result.rows_affected())
    }

    /// Adds a new chapter to the database and returns its new ID.
    /// This function assumes the chapter does not already exist (checked by source_url uniqueness).
    pub async fn add_new_chapter(
        &self,
        series_id: i32,
        chapter_number: f32,
        title: Option<&str>,
        source_url: &str,
    ) -> AnyhowResult<i32> {
        let new_id = sqlx::query_scalar!(
            "INSERT INTO manga_chapters (series_id, chapter_number, title, source_url)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (source_url) DO UPDATE SET updated_at = NOW()
            RETURNING id",
            series_id,
            chapter_number,
            title,
            source_url,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to add chapter with sqlx")?;

        Ok(new_id)
    }

    pub async fn add_chapter_images(
        &self,
        chapter_id: i32,
        image_order: i32,
        image_url: &str, // This will be the R2/CDN Url
    ) -> AnyhowResult<i32> {
        let new_id = sqlx::query_scalar!(
                "INSERT INTO chapter_images (chapter_id, image_order, image_url) VALUES ($1, $2, $3) RETURNING id",
            chapter_id,
            image_order,
            image_url,
            ).fetch_one(&self.pool).await.context("Failed to add chapter image with sqlx")?;

        Ok(new_id)
    }

    pub async fn get_manga_series_by_id(
        &self,
        id: i32,
    ) -> AnyhowResult<Option<MangaSeries>> {
        let series = sqlx::query_as!(
            MangaSeries,
            "SELECT id, title, original_title, description, cover_image_url, current_source_url,
       source_website_host, views_count, bookmarks_count, last_chapter_found_in_storage, processing_status,
       check_interval_minutes, last_checked_at, next_checked_at, created_at, updated_at
       FROM manga_series WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query series by ID with sqlx")?; // Handles cases where no row is found
        Ok(series)
    }

    pub async fn get_manga_series_by_title(
        &self,
        title: &str,
    ) -> AnyhowResult<Option<MangaSeries>> {
        let series = sqlx::query_as!(
            MangaSeries,
            "SELECT id, title, original_title, description, cover_image_url, current_source_url,
            source_website_host, views_count, bookmarks_count, last_chapter_found_in_storage, processing_status,
            check_interval_minutes, last_checked_at, next_checked_at, created_at, updated_at
            FROM manga_series WHERE title = $1",
            title
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query series by title")?;
        Ok(series)
    }

    pub async fn get_paginated_series_with_authors(
        &self,
        page: u32,
        page_size: u32,
    ) -> AnyhowResult<PaginatedResult<SeriesWithAuthors>> {
        let page = page.max(1);

        let limit = page_size as i64;
        let offset = (page as i64 - 1) * limit;

        #[derive(Debug, FromRow)]
        struct QueryResult {
            id: i32,
            title: String,
            original_title: String,
            description: String,
            cover_image_url: String,
            current_source_url: String,
            updated_at: DateTime<Utc>,
            #[sqlx(json)]
            authors: serde_json::Value,
            total_items: Option<i64>,
        }

        let record_list = sqlx::query_as!(
            QueryResult,
            r#"
            SELECT
                ms.id,
                ms.title,
                ms.original_title,
                ms.description,
                ms.cover_image_url,
                ms.current_source_url,
                ms.updated_at,
                -- Use a LEFT JOIN to include all authors, even if they are not linked to the series.
                -- Aggregate author names into a JSON array. If no authors, return an empty array.
                COALESCE(
                    json_agg(a.name) FILTER (WHERE a.id IS NOT NULL),
                    '[]'::json
                ) as "authors!",
                -- The '!' asserts the value is not null, matching COALESCE
                -- Use a window function to get the total count of series without a separate query.
                COUNT(*) OVER () as total_items
            FROM
                manga_series ms
            LEFT JOIN
                manga_authors ma ON ms.id = ma.manga_id
            LEFT JOIN
                authors a ON ma.author_id = a.id
            GROUP BY
                ms.id
            ORDER BY
                ms.updated_at DESC
            LIMIT $1
            OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to query all series")?;

        let total_items = record_list
            .first()
            .map_or(0, |row| row.total_items.unwrap_or(0));

        let series_list = record_list
            .into_iter()
            .map(|r| SeriesWithAuthors {
                id: r.id,
                title: r.title,
                original_title: r.original_title,
                description: r.description,
                cover_image_url: r.cover_image_url,
                current_source_url: r.current_source_url,
                updated_at: r.updated_at,
                authors: serde_json::from_value(r.authors).unwrap_or_default(),
            })
            .collect();

        Ok(PaginatedResult {
            items: series_list,
            total_items,
        })
    }

    pub async fn update_series_last_chapter_found_in_storage(
        &self,
        series_id: i32,
        chapter_number: Option<f32>,
    ) -> AnyhowResult<u64> {
        let result = sqlx::query!(
                "UPDATE manga_series SET last_chapter_found_in_storage = $1, updated_at = NOW() WHERE id = $2",
                chapter_number,
            series_id,
            ).execute(&self.pool).await.context("Failed to update series last chapter found in storage with sqlx")?;

        Ok(result.rows_affected())
    }

    pub async fn update_series_source_urls(
        &self,
        series_id: i32,
        new_source_url: &str,
    ) -> AnyhowResult<u64> {
        let new_host = get_host_from_url(Some(new_source_url));

        let result = sqlx::query!(
                "UPDATE manga_series SET current_source_url = $1, source_website_host = $2, updated_at = NOW() WHERE id = $3",
                new_source_url,
                new_host,
                series_id
            ).execute(&self.pool).await.context("Failed to update series source URLs with sqlx")?;

        Ok(result.rows_affected())
    }

    /// Update the description of a manga series.
    pub async fn update_series_description(
        &self,
        series_id: i32,
        description: &str,
    ) -> AnyhowResult<u64> {
        let result = sqlx::query!(
            "UPDATE manga_series SET description = $1, updated_at = NOW() WHERE id = $2",
            description,
            series_id
        )
        .execute(&self.pool)
        .await
        .context("Failed to update series description with sqlx")?;

        Ok(result.rows_affected())
    }

    /// Updates only the processing status of a series.
    /// Marking a series as "scraping" or "error" without touching check schedules.
    pub async fn update_series_processing_status(
        &self,
        series_id: i32,
        new_status: &str,
    ) -> AnyhowResult<u64> {
        let result = sqlx::query!(
            "UPDATE manga_series SET processing_status = $1, updated_at = NOW() WHERE id = $2",
            new_status,
            series_id,
        )
        .execute(&self.pool)
        .await
        .context("Failed to update series processing status with sqlx")?;

        Ok(result.rows_affected())
    }

    /// Called after a series has been processed
    pub async fn update_series_check_schedule(
        &self,
        series_id: i32,
        new_status: Option<&str>,
        new_next_checked_at: Option<DateTime<Utc>>,
    ) -> AnyhowResult<u64> {
        // First, get the series data asynchronously.
        let series =
            self.get_manga_series_by_id(series_id)
                .await?
                .ok_or_else(|| {
                    anyhow!(
                        "Series with id {} not found for schedule update",
                        series_id
                    )
                })?;

        // Calculate the next check time if not provided
        let final_next_checked_at = new_next_checked_at.unwrap_or_else(|| {
            let mut rng = rand::rng();
            let base_interval = series.check_interval_minutes as i64;
            // Add a random +- 5 minutes jitter to avoid all series checking at the exact same time
            let random_jitter = rng.random_range(-300..=300);
            let actual_interval_secs = (base_interval * 60) + random_jitter;
            Utc::now()
                + chrono::Duration::seconds(actual_interval_secs.max(300))
        });

        let final_status = new_status.unwrap_or(&series.processing_status);

        let result = sqlx::query!(
            "UPDATE manga_series SET processing_status = $1, last_checked_at = NOW(), next_checked_at = $2, updated_at = NOW() WHERE id = $3",
            final_status,
            final_next_checked_at,
            series_id,
            )
            .execute(&self.pool)
            .await
            .context("Failed to update series check schedule with sqlx")?;
        Ok(result.rows_affected())
    }

    pub async fn get_user_by_identifier(
        &self,
        identifier: &str,
    ) -> AnyhowResult<Option<Users>> {
        let user = sqlx::query_as!(
            Users,
                // Check both column email and username
                "SELECT id, username, email, password_hash, role_id FROM users WHERE email = $1 OR username = $1",
                identifier,
            ).fetch_optional(&self.pool).await.context("Failed to get user by identifier")?;
        Ok(user)
    }

    pub async fn get_role_id_by_name(
        &self,
        role_name: &str,
    ) -> AnyhowResult<Option<i32>> {
        let role_id = sqlx::query_scalar!(
            "SELECT id FROM roles WHERE role_name = $1",
            role_name,
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get role ID by name")?;

        Ok(role_id)
    }

    pub async fn get_role_name_by_id(
        &self,
        role_id: i32,
    ) -> AnyhowResult<Option<String>> {
        let role_name = sqlx::query_scalar!(
            "SELECT role_name FROM roles WHERE id = $1",
            role_id,
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get role name by ID")?;

        Ok(role_name)
    }

    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
        role_id: i32,
    ) -> AnyhowResult<i32> {
        let new_user_id = sqlx::query_scalar!(
            "INSERT INTO users (username, email, password_hash, role_id) VALUES ($1, $2, $3, $4) RETURNING id",
            username,
            email,
            password_hash,
            role_id,
        )
            .fetch_one(&self.pool)
            .await
            .context("Failed to create new user")?;

        Ok(new_user_id)
    }

    pub async fn create_password_reset_token(
        &self,
        user_id: i32,
        token: &str,
        expires_at: DateTime<Utc>,
    ) -> AnyhowResult<()> {
        sqlx::query!(
            "INSERT INTO password_reset_tokens (user_id, token, expires_at) VALUES ($1, $2, $3)",
            user_id,
            token,
            expires_at
        )
            .execute(&self.pool)
            .await
            .context("Failed to create password reset token")?;

        Ok(())
    }

    // Retrieves a user's ID and token expiration time by the reset token.
    // Returns a tuple of (user_id, expires_at) if token is found
    pub async fn get_user_by_reset_token(
        &self,
        token: &str,
    ) -> AnyhowResult<Option<(i32, DateTime<Utc>)>> {
        let record = sqlx::query!(
            "SELECT user_id, expires_at FROM password_reset_tokens WHERE token = $1",
            token
        )
            .fetch_optional(&self.pool)
            .await
            .context("Failed to get user by reset token")?
            .map(|row| (row.user_id, row.expires_at));

        Ok(record)
    }

    // Update password hash for a given user ID.
    pub async fn update_user_password_hash_after_reset_password(
        &self,
        user_id: i32,
        new_password_hash: &str,
    ) -> AnyhowResult<()> {
        sqlx::query!(
            "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2",
            new_password_hash,
            user_id
        )
            .execute(&self.pool)
            .await
            .context("Failed to update user password hash")?;

        Ok(())
    }

    pub async fn delete_password_reset_token(
        &self,
        token: &str,
    ) -> AnyhowResult<()> {
        sqlx::query!(
            "DELETE FROM password_reset_tokens WHERE token = $1",
            token
        )
        .execute(&self.pool)
        .await
        .context("Failed to delete password reset token")?;

        Ok(())
    }

    pub async fn delete_chapter_and_images_for_chapter(
        &self,
        series_id: i32,
        chapter_number: f32,
    ) -> AnyhowResult<u64> {
        // exclusive connection from the pool
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to start transaction")?;

        let chapter_id_to_delete = sqlx::query_scalar!(
            "SELECT id FROM manga_chapters WHERE series_id = $1 AND chapter_number = $2",
            series_id,
            chapter_number,
        )
            .fetch_optional(&mut *tx) // Run query inside transaction
            .await
            .context("Failed to get chapter ID to delete")?;

        if let Some(chapter_id) = chapter_id_to_delete {
            sqlx::query!(
                "DELETE FROM chapter_images WHERE chapter_id = $1",
                chapter_id
            )
            .execute(&mut *tx)
            .await
            .context("Failed to delete chapter images")?;

            let result = sqlx::query!(
                "DELETE FROM manga_chapters WHERE id = $1",
                chapter_id
            )
            .execute(&mut *tx)
            .await
            .context("Failed to delete chapter")?;

            // If transaction was successful, commit it
            tx.commit().await.context("Failed to commit transaction")?;

            Ok(result.rows_affected())
        } else {
            Ok(0) // No chapter found to delete
        }
    }

    pub async fn get_images_urls_for_chapter_series(
        &self,
        series_id: i32,
        chapter_number: f32,
    ) -> AnyhowResult<Vec<String>> {
        let urls = sqlx::query_scalar!(
            r#"
            SELECT ci.image_url
            FROM chapter_images ci
            JOIN manga_chapters mc ON ci.chapter_id = mc.id
            WHERE mc.series_id = $1 AND mc.chapter_number = $2
            "#,
            series_id,
            chapter_number,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to get images URLs for chapter series")?;

        Ok(urls)
    }
}

// Retrieves all manhwa series from the database, ordered by title.
/*pub fn get_all_manhwa_series(conn: &Connection) -> Result<Vec<ManhwaSeries>> {
    let mut stmt = conn.prepare("SELECT id, title, current_source_url, source_website_host, last_chapter_found_in_storage, processing_status, check_interval_minutes, last_checked_at, next_checked_at, created_at, updated_at FROM manhwa_series ORDER BY title ASC")?;
    let series_iter = stmt.query_map([], row_manhwa_series)?;

    let mut series_list = Vec::new();
    for series in series_iter {
        series_list.push(series?); // Propagate errors from row mapping
    }
    Ok(series_list)
}*/

// Retrieves manhwa series that are due for checking.
// A series is due if `next_checked_at` is past or NULL, and status is not 'paused' or 'completed'.
/*pub fn get_series_to_check(conn: &Connection, limit: Option<u32>) -> Result<Vec<ManhwaSeries>> {
    let now = current_timestamp();
    // Define statuses to ignore directly in the query for simplicity with params.
    // If statuses were dynamic, building the query string would be more complex.
    let ignore_status = ["paused", "completed"];

    // Simpler query construction using IN operator and binding multiple values
    // This requires enabling `sqlite_parameters_in_query` feature or similar if not default.
    // For rusqlite, we can construct the `?` placeholders dynamically.
    let mut sql = String::from("SELECT id, title, current_source_url, source_website_host, last_chapter_found_in_storage, processing_status, check_interval_minutes, last_checked_at, next_checked_at, created_at, updated_at FROM manhwa_series WHERE (next_check_at <= ?1 OR next_check_at IS NULL) AND processing_status NOT IN (");
    for (i, _) in ignore_status.iter().enumerate() {
        if i > 0 {
            sql.push_str(", ");
        }
        sql.push_str(&format!("?{}", i + 2));
    }
    sql.push_str(") ORDER BY next_checked_at ASC");

    if let Some(l) = limit {
        sql.push_str(&format!(" LIMIT {}", l));
    }

    let mut stmt = conn.prepare(&sql)?;

    // Create dynamic params for the status values
    let mut query_param: Vec<&dyn rusqlite::ToSql> = Vec::new();
    query_param.push(&now);
    for status in &ignore_status {
        query_param.push(status);
    }

    let series_iter = stmt.query_map(query_param.as_slice(), row_manhwa_series)?;

    let mut series_list = Vec::new();
    for series_result in series_iter {
        match series_result {
            Ok(series) => series_list.push(series),
            Err(e) => {
            // Log error and skip this series, or propagate. For now, log and skip.
            eprintln!("[DB] Error mapping series to check: {}", e)
            }
        }
    }
    Ok(series_list)
}*/
