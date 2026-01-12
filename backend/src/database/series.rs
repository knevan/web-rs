use anyhow::Context;
use sqlx::postgres::types::PgInterval;

use super::*;

// =========================================================================
// Public Read Operations
// High traffic, used by standard users/visitors.
// =========================================================================
impl DatabaseService {
    pub async fn get_series_by_id(&self, id: i32) -> AnyhowResult<Option<Series>> {
        let series = sqlx::query_as!(
            Series,
            r#"
            SELECT id, title, original_title, description, cover_image_url, current_source_url,
            source_website_host, views_count, bookmarks_count, total_rating_score, total_ratings_count,
            last_chapter_found_in_storage, processing_status as "processing_status: SeriesStatus",
            check_interval_minutes, last_checked_at, next_checked_at, created_at, updated_at
            FROM series
            WHERE id = $1
            "#,
            id
        )
            .fetch_optional(&self.pool)
            .await
            .context("Failed to query series by ID with sqlx")?;
        Ok(series)
    }

    pub async fn get_series_by_title(&self, title: &str) -> AnyhowResult<Option<Series>> {
        let series = sqlx::query_as!(
            Series,
            r#"
            SELECT id, title, original_title, description, cover_image_url, current_source_url,
            source_website_host, views_count, bookmarks_count, total_rating_score, total_ratings_count,
            last_chapter_found_in_storage, processing_status as "processing_status: SeriesStatus",
            check_interval_minutes, last_checked_at, next_checked_at, created_at, updated_at
            FROM series
            WHERE title = $1
            "#,
            title
        )
            .fetch_optional(&self.pool)
            .await
            .context("Failed to query series by title")?;
        Ok(series)
    }

    // Get authors for a sepecific series
    pub async fn get_authors_by_series_id(&self, series_id: i32) -> AnyhowResult<Vec<String>> {
        let authors_name = sqlx::query_scalar!(
            r#"
            SELECT a.name FROM authors a
            JOIN series_authors sa ON a.id = sa.author_id
            WHERE sa.series_id = $1
            "#,
            series_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to query authors by series ID with sqlx")?;

        Ok(authors_name)
    }

    pub async fn get_category_tag_by_series_id(
        &self,
        series_id: i32,
    ) -> AnyhowResult<Vec<CategoryTag>> {
        let categories = sqlx::query_as!(
            CategoryTag,
            r#"
            SELECT c.id, c.name FROM categories c
            JOIN series_categories sc ON c.id = sc.category_id
            WHERE sc.series_id = $1
            "#,
            series_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to query category tag by series ID with sqlx")?;

        Ok(categories)
    }

    pub async fn get_series_chapters_count(&self, series_id: i32) -> AnyhowResult<i64> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM series_chapters WHERE series_id = $1",
            series_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to get series chapters count")?;

        // It will return a row with 0, not NULL, even if no chapters exist
        Ok(count.unwrap_or(0))
    }

    pub async fn get_list_all_categories(&self) -> AnyhowResult<Vec<CategoryTag>> {
        let categories = sqlx::query_as!(CategoryTag, "SELECT id, name FROM categories")
            .fetch_all(&self.pool)
            .await
            .context("Failed to list all categories with sqlx")?;

        Ok(categories)
    }
}

// =========================================================================
// System, Scraper & Background Worker
// Database locking, scheduling, and automated status updates.
// =========================================================================
impl DatabaseService {
    pub async fn find_and_lock_series_for_check(
        &self,
        limit: i64,
    ) -> AnyhowResult<Vec<SeriesCheckTaskInfo>> {
        let series = sqlx::query_as!(
            SeriesCheckTaskInfo,
            r#"
            WITH candidate AS (
                SELECT id FROM series
                WHERE
                    processing_status = $1
                    AND next_checked_at <= NOW()
                ORDER BY next_checked_at ASC
                LIMIT $2
                FOR UPDATE SKIP LOCKED
            )
            UPDATE series
            SET processing_status = $3
            WHERE id IN (SELECT id FROM candidate)
            RETURNING
                id,
                title,
                current_source_url,
                source_website_host,
                check_interval_minutes
            "#,
            SeriesStatus::Ongoing as _,
            limit,
            SeriesStatus::Processing as _,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to find and lock series for check with sqlx")?;

        Ok(series)
    }

    pub async fn find_and_lock_series_for_job_deletion(
        &self,
    ) -> AnyhowResult<Option<SeriesDeletionJob>> {
        // If the row is already locked by another transaction,
        // it will skip it and look for the next row.
        let series = sqlx::query_as!(
            SeriesDeletionJob,
            r#"
            WITH candidate AS (
                SELECT id FROM series
                WHERE processing_status = $1
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            UPDATE series
            SET processing_status = $2
            WHERE id = (SELECT id FROM candidate)
            RETURNING
                id
            "#,
            SeriesStatus::PendingDeletion as _,
            SeriesStatus::Deleting as _
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find and lock series for job deletion with sqlx")?;

        Ok(series)
    }

    // Called after a series has been checked/processed
    pub async fn update_series_check_schedule(
        &self,
        series_id: i32,
        check_interval_minutes: i32,
        new_status: SeriesStatus,
        new_next_checked_at: Option<DateTime<Utc>>,
    ) -> AnyhowResult<u64> {
        // Calculate the next check time if not provided
        let final_next_checked_at = new_next_checked_at.unwrap_or_else(|| {
            let mut rng = rand::rng();
            let base_interval = check_interval_minutes as i64;
            // Add a random +- 5 minutes jitter to avoid all series checking at the exact same time
            let random_jitter = rng.random_range(-300..=300);
            let actual_interval_secs = (base_interval * 60) + random_jitter;
            Utc::now() + chrono::Duration::seconds(actual_interval_secs.max(300))
        });

        let result = sqlx::query!(
            "UPDATE series
            SET processing_status = $1, last_checked_at = NOW(), next_checked_at = $2 WHERE id = $3",
            new_status as _,
            final_next_checked_at,
            series_id,
            )
            .execute(&self.pool)
            .await
            .context("Failed to update series check schedule with sqlx")?;
        Ok(result.rows_affected())
    }

    /// Updates only the processing status of a series.
    /// Marking a series as "scraping" or "error" without touching check schedules.
    pub async fn update_series_processing_status(
        &self,
        series_id: i32,
        new_status: SeriesStatus,
    ) -> AnyhowResult<u64> {
        let result = sqlx::query!(
            "UPDATE series SET processing_status = $1, updated_at = NOW() WHERE id = $2",
            new_status as _,
            series_id,
        )
        .execute(&self.pool)
        .await
        .context("Failed to update series processing status with sqlx")?;

        Ok(result.rows_affected())
    }

    pub async fn update_series_last_chapter_found_in_storage(
        &self,
        series_id: i32,
        chapter_number: f32,
    ) -> AnyhowResult<u64> {
        let result = sqlx::query!(
                "UPDATE series
                SET last_chapter_found_in_storage = GREATEST(COALESCE(last_chapter_found_in_storage, 0), $1),
                    updated_at = NOW()
                WHERE id = $2",
                chapter_number,
            series_id,
            ).execute(&self.pool).await.context("Failed to update series last chapter found in storage with sqlx")?;

        Ok(result.rows_affected())
    }

    // Called only if there's new valid content (new chapter)
    pub async fn update_series_new_content_timestamp(&self, series_id: i32) -> AnyhowResult<u64> {
        let result = sqlx::query!(
            "UPDATE series SET updated_at = NOW() WHERE id = $1",
            series_id,
        )
        .execute(&self.pool)
        .await
        .context("Failed to update `updated_at` timestamp")?;

        Ok(result.rows_affected())
    }

    pub async fn get_image_keys_for_series_deletion(
        &self,
        series_id: i32,
    ) -> AnyhowResult<Option<SeriesDeletionImagekeys>> {
        let cover_url = sqlx::query_scalar!(
            "SELECT cover_image_url FROM series WHERE id = $1",
            series_id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get cover image URL")?;

        let cover_image_url = match cover_url {
            Some(url) => Some(url),
            None => return Ok(None),
        };

        let chapter_image_urls = sqlx::query_scalar!(
            r#"
            SELECT ci.image_url
            FROM chapter_images ci
            JOIN series_chapters sc ON ci.chapter_id = sc.id
            WHERE sc.series_id = $1
            "#,
            series_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to get chapter image URLs")?;

        Ok(Some(SeriesDeletionImagekeys {
            cover_image_url,
            chapter_image_urls,
        }))
    }

    // Query helper for delete old view logs
    pub async fn cleanup_old_view_logs(&self) -> AnyhowResult<u64> {
        let retention_interval = PgInterval {
            months: 0,
            days: 35,
            microseconds: 0,
        };

        let result = sqlx::query!(
            "DELETE FROM series_view_log WHERE viewed_at < NOW() - $1::interval",
            retention_interval as _
        )
        .execute(&self.pool)
        .await
        .context("Failed to cleanup old view logs with sqlx")?;

        Ok(result.rows_affected())
    }
}
