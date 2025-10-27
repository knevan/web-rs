use super::*;

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
impl DatabaseService {
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
            "INSERT INTO series_chapters (series_id, chapter_number, title, source_url)
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
            "SELECT id FROM series_chapters WHERE series_id = $1 AND chapter_number = $2",
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

            let result =
                sqlx::query!("DELETE FROM series_chapters WHERE id = $1", chapter_id)
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
