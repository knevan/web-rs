use super::*;

// =========================================================================
// Public Read Used by the API to display chapters to users
// =========================================================================
impl DatabaseService {
    pub async fn get_images_urls_for_chapter_series(
        &self,
        series_id: i32,
        chapter_number: f32,
    ) -> AnyhowResult<Vec<String>> {
        let urls = sqlx::query_scalar!(
            r#"
            SELECT ci.image_url
            FROM chapter_images ci
            JOIN series_chapters mc ON ci.chapter_id = mc.id
            WHERE mc.series_id = $1 AND mc.chapter_number = $2
            ORDER BY ci.image_order ASC
            "#,
            series_id,
            chapter_number,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to get images URLs for chapter series")?;

        Ok(urls)
    }

    // Get chapters for a sepecific series
    pub async fn get_chapters_by_series_id(
        &self,
        series_id: i32,
    ) -> AnyhowResult<Vec<SeriesChapter>> {
        let chapters = sqlx::query_as!(
            SeriesChapter,
            r#"
            SELECT id, series_id, chapter_number, status AS "status: _",title, source_url, created_at
            FROM series_chapters
            WHERE series_id = $1
            ORDER BY chapter_number
            DESC
            "#,
            series_id
        )
            .fetch_all(&self.pool)
            .await
            .context("Failed to query chapters by series ID with sqlx")?;

        Ok(chapters)
    }
}

// =========================================================================
// Scraper Ingestion & Data Entry
// =========================================================================
impl DatabaseService {
    /// Adds a new chapter to the database and returns its new ID.
    /// This function assumes the chapter does not already exist (checked by source_url uniqueness).
    pub async fn add_new_chapter(
        &self,
        series_id: i32,
        chapter_number: f32,
        title: Option<&str>,
        source_url: &str,
        chapter_status: ChapterStatus,
    ) -> AnyhowResult<i32> {
        let new_id = sqlx::query_scalar!(
            "INSERT INTO series_chapters (series_id, chapter_number, title, source_url, status)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (series_id, chapter_number)
            DO UPDATE SET
                updated_at = NOW(),
                source_url = EXCLUDED.source_url,
                status = EXCLUDED.status
            RETURNING id",
            series_id,
            chapter_number,
            title,
            source_url,
            chapter_status as ChapterStatus
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
        image_url: &str,
    ) -> AnyhowResult<i32> {
        let new_id = sqlx::query_scalar!(
            "INSERT INTO chapter_images (chapter_id, image_order, image_url) VALUES ($1, $2, $3) RETURNING id",
            chapter_id,
            image_order,
            image_url,
            )
            .fetch_one(&self.pool)
            .await
            .context("Failed to add chapter image with sqlx")?;

        Ok(new_id)
    }

    pub async fn get_max_known_chapter(&self, series_id: i32) -> AnyhowResult<f32> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT MAX(chapter_number)
            FROM series_chapters
            WHERE series_id = $1
            "#,
            series_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to get max known chapter number")?;

        Ok(result.unwrap_or(0.0))
    }
}

// =========================================================================
// Background Worker & Job Queue
// Handling "Processing" status, locking, and job distribution
// =========================================================================
impl DatabaseService {
    pub async fn find_and_lock_pending_chapters(
        &self,
        limit: i64,
    ) -> AnyhowResult<Vec<DownloadJobData>> {
        let record = sqlx::query_as!(
            DownloadJobData,
            r#"
            WITH locked_rows AS (
                SELECT id
                FROM series_chapters
                WHERE status = 'Processing'
                ORDER BY created_at ASC
                LIMIT $1
                FOR UPDATE SKIP LOCKED
             )
            UPDATE series_chapters sc
            SET
                status = 'Processing',
                updated_at = NOW()
            FROM locked_rows lr, series s
            WHERE sc.id = lr.id AND sc.series_id = s.id
            RETURNING
                sc.id as chapter_id,
                sc.chapter_number,
                sc.source_url as chapter_url,
                s.id as series_id,
                s.title as series_title,
                s.source_website_host as source_host,
                s.current_source_url as series_url
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to deque pending chapters")?;

        Ok(record)
    }

    pub async fn update_chapter_status(
        &self,
        chapter_id: i32,
        new_status: ChapterStatus,
    ) -> AnyhowResult<u64> {
        let result = sqlx::query!(
            "UPDATE series_chapters SET status = $1 WHERE id = $2",
            new_status as _,
            chapter_id,
        )
        .execute(&self.pool)
        .await;

        match result {
            Ok(res) => Ok(res.rows_affected()),
            Err(e) => {
                // Log error sqlx yang detail di sini
                eprintln!(
                    "[DB_ERROR] Failed to update chapter status for ID {}: {:?}",
                    chapter_id, e
                );
                // Kembalikan error agar ? tetap berfungsi
                Err(anyhow::anyhow!(e).context("Failed to update status chapter"))
            }
        }
    }
}
