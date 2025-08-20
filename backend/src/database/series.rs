use super::*;
use anyhow::{Context, anyhow};
use sqlx::postgres::types::PgInterval;

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
    pub async fn add_new_series(
        &self,
        data: &NewSeriesData<'_>,
    ) -> AnyhowResult<i32> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to begin transaction")?;

        let host = get_host_from_url(Some(data.source_url));

        let new_series_id = sqlx::query_scalar!(
            r#"INSERT INTO series
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
                    "INSERT INTO series_authors (series_id, author_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                    new_series_id,
                    author_id
                ).execute(&mut *tx).await.context(format!("Failed to link author {} to manga", name))?;
            }
        }

        if let Some(category_ids) = data.category_ids
            && !category_ids.is_empty()
        {
            for &category_id in category_ids {
                // Insert the relationship into the series_categories junction table.
                sqlx::query!(
                        "INSERT INTO series_categories (series_id, category_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                        new_series_id,
                        category_id
                    )
                        .execute(&mut *tx)
                        .await
                        .context(format!("Failed to link category {} to manga", category_id))?;
            }
        }

        tx.commit().await.context("Failed to commit transaction")?;

        Ok(new_series_id)
    }

    pub async fn update_series_metadata(
        &self,
        series_id: i32,
        data: &UpdateSeriesData<'_>,
    ) -> AnyhowResult<u64> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to begin transaction")?;

        let host = get_host_from_url(data.source_url);

        let result = sqlx::query!(
            "UPDATE series
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
                "DELETE FROM series_authors WHERE series_id = $1",
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
                    "INSERT INTO series_authors (series_id, author_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                    series_id,
                    author_id
                )
                    .execute(&mut *tx)
                    .await
                    .context(format!("Failed to link author {} to manga", name))?;
            }
        }

        if let Some(category_ids) = data.category_ids {
            sqlx::query!(
                "DELETE FROM series_categories WHERE series_id = $1",
                series_id
            )
            .execute(&mut *tx)
            .await
            .context("Failed to delete existing categories for manga")?;

            if !category_ids.is_empty() {
                for category_id in category_ids {
                    sqlx::query!(
                        "INSERT INTO series_categories (series_id, category_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                        series_id,
                        category_id
                    )
                        .execute(&mut *tx)
                    .await
                    .context(format!("Failed to link category {} to manga", category_id))?;
                }
            }
        }

        tx.commit().await.context("Failed to commit transaction")?;

        Ok(result.rows_affected())
    }

    pub async fn get_manga_series_by_id(
        &self,
        id: i32,
    ) -> AnyhowResult<Option<Series>> {
        let series = sqlx::query_as!(
            Series,
            "SELECT id, title, original_title, description, cover_image_url, current_source_url,
       source_website_host, views_count, bookmarks_count, last_chapter_found_in_storage, processing_status,
       check_interval_minutes, last_checked_at, next_checked_at, created_at, updated_at
       FROM series WHERE id = $1",
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
    ) -> AnyhowResult<Option<Series>> {
        let series = sqlx::query_as!(
            Series,
            "SELECT id, title, original_title, description, cover_image_url, current_source_url,
            source_website_host, views_count, bookmarks_count, last_chapter_found_in_storage, processing_status,
            check_interval_minutes, last_checked_at, next_checked_at, created_at, updated_at
            FROM series WHERE title = $1",
            title
        )
            .fetch_optional(&self.pool)
            .await
            .context("Failed to query series by title")?;
        Ok(series)
    }

    // Get authors for a sepecific series
    pub async fn get_authors_by_series_id(
        &self,
        series_id: i32,
    ) -> AnyhowResult<Vec<String>> {
        let authors_name = sqlx::query_scalar!(
            r#"SELECT a.name FROM authors a
            JOIN series_authors sa ON a.id = sa.author_id
            WHERE sa.series_id = $1"#,
            series_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to query authors by series ID with sqlx")?;

        Ok(authors_name)
    }

    // Get chapters for a sepecific series
    pub async fn get_chapters_by_series_id(
        &self,
        series_id: i32,
    ) -> AnyhowResult<Vec<SeriesChapter>> {
        let chapters = sqlx::query_as!(
            SeriesChapter,
            "SELECT id, series_id, chapter_number, title, source_url, created_at FROM series_chapters WHERE series_id = $1 ORDER BY chapter_number DESC",
            series_id
        )
            .fetch_all(&self.pool)
            .await
            .context("Failed to query chapters by series ID with sqlx")?;

        Ok(chapters)
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
            WHERE sc.series_id = $1"#,
            series_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to query category tag by series ID with sqlx")?;

        Ok(categories)
    }

    // Get paginated series list for admin
    pub async fn get_admin_paginated_series(
        &self,
        page: u32,
        page_size: u32,
        search_query: Option<&str>,
    ) -> AnyhowResult<PaginatedResult<SeriesWithAuthors>> {
        let page = page.max(1);

        let limit = page_size as i64;
        let offset = (page as i64 - 1) * limit;

        // Format search query, wraps the query in '%' to allow match substrings
        let formatted_search_query = search_query.map(|s| format!("%{}%", s));

        #[derive(Debug, FromRow)]
        struct QueryResult {
            id: i32,
            title: String,
            original_title: String,
            description: String,
            cover_image_url: String,
            current_source_url: String,
            updated_at: DateTime<Utc>,
            processing_status: String,
            #[sqlx(json)]
            authors: serde_json::Value,
            total_items: Option<i64>,
        }

        let record_list = sqlx::query_as!(
            QueryResult,
            r#"
            SELECT
                sr.id,
                sr.title,
                sr.original_title,
                sr.description,
                sr.cover_image_url,
                sr.current_source_url,
                sr.updated_at,
                sr.processing_status,
                -- Use a LEFT JOIN to include all authors, even if they are not linked to the series.
                -- Aggregate author names into a JSON array. If no authors, return an empty array.
                -- The '!' asserts the value is not null, matching COALESCE
                COALESCE(
                    json_agg(a.name) FILTER (WHERE a.id IS NOT NULL),
                    '[]'::json
                ) as "authors!",
                -- Use a window function to get the total count of series without a separate query.
                COUNT(*) OVER () as total_items
            FROM
                series sr
            LEFT JOIN
                series_authors sa ON sr.id = sa.series_id
            LEFT JOIN
                authors a ON sa.author_id = a.id
            WHERE
                -- Filter by search query if provided
                ($3::TEXT IS NULL OR sr.title ILIKE $3)
            GROUP BY
                sr.id
            ORDER BY
                sr.updated_at DESC
            LIMIT $1
            OFFSET $2
            "#,
            limit,
            offset,
            formatted_search_query
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
                processing_status: r.processing_status,
                updated_at: r.updated_at,
                authors: serde_json::from_value(r.authors).unwrap_or_default(),
            })
            .collect();

        Ok(PaginatedResult {
            items: series_list,
            total_items,
        })
    }

    pub async fn get_public_series_paginated(
        &self,
        page: u32,
        page_size: u32,
        order_by: SeriesOrderBy,
    ) -> AnyhowResult<Vec<SeriesWithAuthors>> {
        let limit = page_size as i64;
        let offset = (page.max(1) as i64 - 1) * limit;

        // Ordering column
        let order_by_clause = match order_by {
            SeriesOrderBy::CreatedAt => "sr.created_at",
            SeriesOrderBy::UpdatedAt => "sr.updated_at",
        };

        let query_string = format!(
            r#"
            SELECT
                sr.id,
                sr.title,
                sr.original_title,
                sr.description,
                sr.cover_image_url,
                sr.current_source_url,
                sr.updated_at,
                sr.processing_status,
                COALESCE(
                    json_agg(a.name) FILTER (WHERE a.id IS NOT NULL),
                    '[]'::json
                ) as authors
            FROM
                series sr
            LEFT JOIN
                series_authors sa ON sr.id = sa.series_id
            LEFT JOIN
                authors a ON sa.author_id = a.id
            WHERE
                sr.processing_status = 'Ongoing'
            GROUP BY
                sr.id
            ORDER BY
                {} DESC
            LIMIT $1
            OFFSET $2
            "#,
            order_by_clause
        );

        let series_list = sqlx::query_as::<_, SeriesWithAuthors>(&query_string)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .context("Failed to query public series")?;

        Ok(series_list)
    }

    /// Updates only the processing status of a series.
    /// Marking a series as "scraping" or "error" without touching check schedules.
    pub async fn update_series_processing_status(
        &self,
        series_id: i32,
        new_status: &str,
    ) -> AnyhowResult<u64> {
        let result = sqlx::query!(
            "UPDATE series SET processing_status = $1, updated_at = NOW() WHERE id = $2",
            new_status,
            series_id,
        )
            .execute(&self.pool)
            .await
            .context("Failed to update series processing status with sqlx")?;

        Ok(result.rows_affected())
    }

    // Called only if there's new valid content (new chapter)
    pub async fn update_series_new_content_timestamp(
        &self,
        series_id: i32,
    ) -> AnyhowResult<u64> {
        let result = sqlx::query!(
            "UPDATE series SET updated_at = NOW() WHERE id = $1",
            series_id,
        )
        .execute(&self.pool)
        .await
        .context("Failed to update `updated_at` timestamp")?;

        Ok(result.rows_affected())
    }

    // Called after a series has been processed
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
            "UPDATE series SET processing_status = $1, last_checked_at = NOW(), next_checked_at = $2 WHERE id = $3",
            final_status,
            final_next_checked_at,
            series_id,
            )
            .execute(&self.pool)
            .await
            .context("Failed to update series check schedule with sqlx")?;
        Ok(result.rows_affected())
    }

    pub async fn update_series_last_chapter_found_in_storage(
        &self,
        series_id: i32,
        chapter_number: Option<f32>,
    ) -> AnyhowResult<u64> {
        let result = sqlx::query!(
                "UPDATE series SET last_chapter_found_in_storage = $1, updated_at = NOW() WHERE id = $2",
                chapter_number,
            series_id,
            ).execute(&self.pool).await.context("Failed to update series last chapter found in storage with sqlx")?;

        Ok(result.rows_affected())
    }

    pub async fn get_series_chapters_count(
        &self,
        series_id: i32,
    ) -> AnyhowResult<i64> {
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

    pub async fn delete_series_by_id(
        &self,
        series_id: i32,
    ) -> AnyhowResult<u64> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to start transaction for series deletion")?;

        let chapter_ids: Vec<i32> = sqlx::query_scalar!(
            "SELECT id FROM series_chapters WHERE series_id = $1",
            series_id
        )
        .fetch_all(&mut *tx)
        .await
        .context("Failed to get chapter IDs for deletion")?;

        if !chapter_ids.is_empty() {
            // Delete all image record for all chapters
            sqlx::query!(
                "DELETE FROM chapter_images WHERE chapter_id = ANY ($1)",
                &chapter_ids
            )
            .execute(&mut *tx)
            .await
            .context("Failed to delete chapter images")?;
        }

        // Delete all chapter records
        sqlx::query!(
            "DELETE FROM series_chapters WHERE series_id = $1",
            series_id
        )
        .execute(&mut *tx)
        .await
        .context("Failed to delete series chapters")?;

        // Delete all author link records
        sqlx::query!(
            "DELETE FROM series_authors WHERE series_id = $1",
            series_id
        )
        .execute(&mut *tx)
        .await
        .context("Failed to delete series-authors links")?;

        let result =
            sqlx::query!("DELETE FROM series WHERE id = $1", series_id)
                .execute(&mut *tx)
                .await
                .context("Failed to delete series")?;

        tx.commit()
            .await
            .context("Failed to commit transaction for series deletion")?;

        Ok(result.rows_affected())
    }

    pub async fn mark_series_for_deletion(
        &self,
        series_id: i32,
    ) -> AnyhowResult<u64> {
        let result = sqlx::query!(
            "UPDATE series SET processing_status = 'pending_deletion',
                  updated_at = NOW() WHERE id = $1 AND processing_status NOT IN ('pending_deletion', 'deleting')",
            series_id
        )
            .execute(&self.pool)
            .await
            .context("Failed to mark series for deletion with sqlx")?;

        Ok(result.rows_affected())
    }

    pub async fn find_and_lock_series_for_check(
        &self,
    ) -> AnyhowResult<Option<Series>> {
        let series = sqlx::query_as!(
            Series,
            r#"
            WITH candidate AS (
                SELECT id FROM series
                WHERE
                    processing_status = 'Ongoing'
                    AND next_checked_at <= NOW()
                ORDER BY next_checked_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            UPDATE series
            SET processing_status = 'processing'
            WHERE id = (SELECT id FROM candidate)
            RETURNING
                id, title, original_title, description, cover_image_url, current_source_url,
                source_website_host, views_count, bookmarks_count, last_chapter_found_in_storage,
                'processing' as "processing_status!", check_interval_minutes, last_checked_at,
                next_checked_at, created_at, updated_at
            "#
        )
            .fetch_optional(&self.pool)
            .await
            .context("Failed to find and lock series for check with sqlx")?;

        Ok(series)
    }

    pub async fn find_and_lock_series_for_job_deletion(
        &self,
    ) -> AnyhowResult<Option<Series>> {
        // If the row is already locked by another transaction,
        // it will skip it and look for the next row.
        let series = sqlx::query_as!(
            Series,
            r#"
            WITH candidate AS (
                SELECT id FROM series
                WHERE processing_status = 'pending_deletion'
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            UPDATE series
            SET processing_status = 'deleting'
            WHERE id = (SELECT id FROM candidate)
            RETURNING
                id, title, original_title, description, cover_image_url, current_source_url,
                source_website_host, views_count, bookmarks_count, last_chapter_found_in_storage,
                'deleting' as "processing_status!", check_interval_minutes, last_checked_at,
                next_checked_at, created_at, updated_at
            "#
        )
            .fetch_optional(&self.pool)
            .await
            .context("Failed to find and lock series for job deletion with sqlx")?;

        Ok(series)
    }

    pub async fn create_category_tag(
        &self,
        name: &str,
    ) -> AnyhowResult<CategoryTag> {
        let category = sqlx::query_as!(
            CategoryTag,
            "INSERT INTO categories (name) VALUES ($1) RETURNING id, name",
            name
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to create category tag with sqlx")?;

        Ok(category)
    }

    pub async fn delete_category_tag(&self, id: i32) -> AnyhowResult<u64> {
        let result = sqlx::query!("DELETE FROM categories WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .context("Failed to delete category tag with sqlx")?;

        Ok(result.rows_affected())
    }

    pub async fn get_list_all_categories(
        &self,
    ) -> AnyhowResult<Vec<CategoryTag>> {
        let categories =
            sqlx::query_as!(CategoryTag, "SELECT id, name FROM categories")
                .fetch_all(&self.pool)
                .await
                .context("Failed to list all categories with sqlx")?;

        Ok(categories)
    }

    pub async fn record_series_view(&self, series_id: i32) -> AnyhowResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to begin transaction.")?;

        sqlx::query!(
            "INSERT INTO series_view_log (series_id) VALUES ($1)",
            series_id
        )
        .execute(&mut *tx)
        .await
        .context("Failed to record series view with sqlx")?;

        // Increment the total view count
        sqlx::query!(
            "UPDATE series SET views_count = views_count + 1 WHERE id = $1",
            series_id
        )
        .execute(&mut *tx)
        .await
        .context("Failed to increment series view count with sqlx")?;

        tx.commit().await.context("Failed to commit transaction.")?;

        Ok(())
    }

    pub async fn fetch_most_viewed_series(
        &self,
        period: &str,
        limit: i64,
    ) -> AnyhowResult<Vec<MostViewedSeries>> {
        let pg_interval = match period {
            "1 hour" => PgInterval {
                months: 0,
                days: 0,
                microseconds: 3_600_000_000,
            },
            "1 day" => PgInterval {
                months: 0,
                days: 1,
                microseconds: 0,
            },
            "1 week" => PgInterval {
                months: 0,
                days: 7,
                microseconds: 0,
            },
            "1 month" => PgInterval {
                months: 1,
                days: 0,
                microseconds: 0,
            },
            // Default to 1 day
            _ => PgInterval {
                months: 0,
                days: 1,
                microseconds: 0,
            },
        };

        let series_list = sqlx::query_as!(
            MostViewedSeries,
            r#"
            SELECT
                s.id,
                s.title,
                s.cover_image_url,
                COUNT(svl.series_id) AS "view_count"
            FROM
                series s
            INNER JOIN
                series_view_log svl ON s.id = svl.series_id
            WHERE
                svl.viewed_at >= NOW() - $1::interval
            GROUP BY
                s.id
            ORDER BY
                view_count DESC
            LIMIT $2
            "#,
            pg_interval,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch most viewed series with sqlx")?;

        Ok(series_list)
    }

    pub async fn cleanup_old_view_logs(&self) -> AnyhowResult<u64> {
        let retention_interval = PgInterval {
            months: 1,
            days: 0,
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

    pub async fn add_bookmarked_series(
        &self,
        user_id: i32,
        series_id: i32,
    ) -> AnyhowResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to begin transaction.")?;

        // Insert new bookmark record
        sqlx::query!(
            "INSERT INTO user_bookmarks (user_id, series_id) VALUES ($1, $2) ON CONFLICT (user_id, series_id) DO NOTHING",
            user_id,
            series_id
        )
            .execute(&mut *tx)
        .await
        .context("Failed to add bookmarked series view with sqlx")?;

        // Increment bookmark count on the series table
        sqlx::query!(
            "UPDATE series SET bookmarks_count = bookmarks_count + 1 WHERE id = $1",
            series_id
        )
        .execute(&mut *tx)
        .await
        .context("Failed to update bookmarked series view with sqlx")?;

        tx.commit().await.context("Failed to commit transaction.")?;

        Ok(())
    }

    pub async fn delete_bookmarked_series(
        &self,
        user_id: i32,
        series_id: i32,
    ) -> AnyhowResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to begin transaction.")?;

        // Delete the bookmark record.
        let delete_result = sqlx::query!(
            "DELETE FROM user_bookmarks WHERE user_id = $1 AND series_id = $2",
            user_id,
            series_id
        )
        .execute(&mut *tx)
        .await
        .context("Failed to deleted bookmark")?;

        // Only decrement counter if row actually deleted to prevent counts from going negative
        if delete_result.rows_affected() > 0 {
            sqlx::query!(
                "UPDATE series SET bookmarks_count = GREATEST (0, bookmarks_count - 1) WHERE id = $1",
                series_id
            )
            .execute(&mut *tx)
            .await
            .context("Failed to decrement serues bookmark count with sqlx")?;
        }

        tx.commit().await.context("Failed to commit transaction.")?;

        Ok(())
    }

    pub async fn is_series_bookmarked(
        &self,
        user_id: i32,
        series_id: i32,
    ) -> AnyhowResult<bool> {
        // Query to check for existence of a bookmark entry
        let exist = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM user_bookmarks WHERE user_id = $1 AND series_id = $2)",
            user_id,
            series_id
        )
            .fetch_one(&self.pool)
            .await
            .context("Failed to check bookmarked series view with sqlx")?;

        Ok(exist.unwrap_or(false))
    }

    pub async fn get_bookmarked_series_for_user(
        &self,
        user_id: i32,
    ) -> AnyhowResult<Vec<BookmarkedSeries>> {
        let series_list = sqlx::query_as!(
            BookmarkedSeries,
            r#"
            SELECT
                s.id,
                s.title,
                s.cover_image_url,
                s.updated_at,
                s.last_chapter_found_in_storage,
                sc.title as chapter_title
            FROM
                user_bookmarks ub
            JOIN
                series s ON ub.series_id = s.id
            LEFT JOIN
                series_chapters sc ON s.id = sc.series_id
                AND s.last_chapter_found_in_storage = sc.chapter_number
            WHERE
                ub.user_id = $1
            ORDER BY
                s.updated_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch bookmarked series list")?;

        Ok(series_list)
    }
}
