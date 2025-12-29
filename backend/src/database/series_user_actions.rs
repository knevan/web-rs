use anyhow::Context;
use sqlx::postgres::types::PgInterval;
use sqlx::QueryBuilder;

use super::*;

/// Macros `sqlx::query!`
/// For DML operations (INSERT, UPDATE, DELETE) or SELECT,
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
    pub async fn record_series_view(&self, series_id: i32) -> AnyhowResult<()> {
        sqlx::query!(
            r#"
            WITH log_view_insert AS (
                INSERT INTO series_view_log (series_id) VALUES ($1)
            )
            UPDATE series
            SET views_count = views_count + 1
            WHERE id = $1
            "#,
            series_id
        )
        .execute(&self.pool)
        .await
        .context("Failed to record series view with sqlx")?;

        Ok(())
    }

    // fetch most viewed series
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

    pub async fn add_bookmarked_series(&self, user_id: i32, series_id: i32) -> AnyhowResult<()> {
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

    pub async fn delete_bookmarked_series(&self, user_id: i32, series_id: i32) -> AnyhowResult<()> {
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
                .context("Failed to decrement series bookmark count with sqlx")?;
        }

        tx.commit().await.context("Failed to commit transaction.")?;

        Ok(())
    }

    pub async fn is_series_bookmarked(&self, user_id: i32, series_id: i32) -> AnyhowResult<bool> {
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

    pub async fn add_or_update_series_rating(
        &self,
        series_id: i32,
        rating: i16,
        user_id: i32,
    ) -> AnyhowResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to begin transaction.")?;

        let old_rating: Option<i16> = sqlx::query_scalar!(
            "SELECT rating FROM series_ratings WHERE user_id = $1 AND series_id = $2",
            user_id,
            series_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO series_ratings (series_id, user_id, rating) VALUES ($1, $2, $3)
            ON CONFLICT (user_id, series_id) DO UPDATE SET rating = $3, updated_at = NOW()
            "#,
            series_id,
            user_id,
            rating,
        )
        .execute(&mut *tx)
        .await
        .context("Failed to update series rating")?;

        match old_rating {
            Some(old_score) => {
                // User is UPDATING their rating
                let rating_diff = rating as i64 - old_score as i64;
                sqlx::query!(
                    "UPDATE series SET total_rating_score = total_rating_score + $1 WHERE id = $2",
                    rating_diff,
                    series_id,
                )
                .execute(&mut *tx)
                .await
                .context("Failed to update user series rating")?;
            }
            None => {
                // New rating
                sqlx::query!(
                    "UPDATE series SET total_rating_score = total_rating_score + $1, total_ratings_count = total_ratings_count + 1 WHERE id = $2",
                    rating as i64,
                    series_id,
                )
                    .execute(&mut *tx)
                    .await
                    .context("Failed to update new series rating")?;
            }
        }
        tx.commit().await.context("Failed to commit transaction.")?;
        Ok(())
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
            SeriesOrderBy::ViewsCount => "sr.views_count",
            SeriesOrderBy::Rating => "sr.total_rating_score",
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

    // Paginated fetching of latest release series
    pub async fn get_latest_release_series_chapter_paginated(
        &self,
        page: u32,
        page_size: u32,
    ) -> AnyhowResult<PaginatedResult<LatestReleaseSeries>> {
        let limit = page_size.min(100) as i64;
        let offset = (page.max(1) as i64 - 1) * limit;

        #[derive(Debug, FromRow)]
        struct QueryResult {
            id: i32,
            title: String,
            original_title: Option<String>,
            #[sqlx(json)]
            authors: serde_json::Value,
            cover_image_url: String,
            description: String,
            last_chapter_found_in_storage: Option<f32>,
            updated_at: DateTime<Utc>,
            chapter_title: Option<String>,
            total_items: Option<i64>,
        }

        let records = sqlx::query_as!(
            QueryResult,
            r#"
            SELECT
                s.id,
                s.title,
                s.original_title,
                s.description,
                s.cover_image_url,
                s.updated_at,
                s.last_chapter_found_in_storage,
                sc.title as chapter_title,
                COALESCE(json_agg(DISTINCT a.name ORDER BY a.name) FILTER (WHERE a.id IS NOT NULL),
                    '[]'::json) as authors,
                COUNT(*) OVER () as total_items
            FROM
                series s
            LEFT JOIN
                series_chapters sc ON s.id = sc.series_id
                AND s.last_chapter_found_in_storage = sc.chapter_number
            LEFT JOIN
                    series_authors sa ON s.id = sa.series_id
            LEFT JOIN
                    authors a ON sa.author_id = a.id
            WHERE
                s.updated_at >= NOW() - interval '7 days'
            GROUP BY
                s.id, sc.title
            ORDER BY
                s.updated_at DESC
            LIMIT $1
            OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to query latest release series")?;

        let total_items = records
            .first()
            .map_or(0, |row| row.total_items.unwrap_or(0));

        let series_list = records
            .into_iter()
            .map(|r| LatestReleaseSeries {
                id: r.id,
                title: r.title,
                original_title: r.original_title,
                authors: r.authors,
                description: r.description,
                cover_image_url: r.cover_image_url,
                last_chapter_found_in_storage: r.last_chapter_found_in_storage,
                updated_at: r.updated_at,
                chapter_title: r.chapter_title,
            })
            .collect();

        Ok(PaginatedResult {
            items: series_list,
            total_items,
        })
    }

    // Paginated fetching of browse series with/without filters
    pub async fn browse_series_paginated_with_filters(
        &self,
        page: u32,
        page_size: u32,
        order_by: SeriesOrderBy,
        include_category_ids: &[i32],
        exclude_category_ids: &[i32],
        search_query: Option<&str>,
    ) -> AnyhowResult<PaginatedResult<BrowseSeriesSearchResult>> {
        let page_size = page_size.min(100);
        let limit = page_size as i64;
        let offset = (page.max(1) as i64 - 1) * limit;

        let order_by_clause = match order_by {
            SeriesOrderBy::CreatedAt => "s.created_at DESC",
            SeriesOrderBy::UpdatedAt => "s.updated_at DESC",
            SeriesOrderBy::ViewsCount => "s.views_count DESC",
            SeriesOrderBy::Rating => "s.total_rating_score DESC",
        };

        let has_include_filters = !include_category_ids.is_empty();
        let has_exclude_filters = !exclude_category_ids.is_empty();

        const SIMILARITY_THRESHOLD: f32 = 0.20;
        let trimmed_search_query = search_query
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim());

        // Define common parts of the query
        const SELECT_FIELD: &str = r#"
            SELECT
                s.id, s.title, s.original_title, s.description, s.cover_image_url,
                s.updated_at, s.last_chapter_found_in_storage,
                COALESCE(json_agg(DISTINCT a.name ORDER BY a.name) FILTER (WHERE a.id IS NOT NULL), '[]'::json) as authors,
                COALESCE(json_agg(DISTINCT c.name ORDER BY c.name) FILTER (WHERE c.id IS NOT NULL), '[]'::json) as categories,
                COUNT(*) OVER () AS total_items
        "#;

        // Join table
        const JOIN_LOGIC: &str = r#"
            LEFT JOIN series_authors sa ON s.id = sa.series_id
            LEFT JOIN authors a ON sa.author_id = a.id
            LEFT JOIN series_categories sc ON s.id = sc.series_id
            LEFT JOIN categories c ON sc.category_id = c.id
        "#;

        // This GROUP BY must include all non-aggregated columns from SELECT_FIELDS
        const GROUP_BY_LOGIC: &str = r#"
            GROUP BY s.id, s.title, s.original_title, s.description, s.cover_image_url,
                 s.updated_at, s.last_chapter_found_in_storage, s.views_count,
                 s.total_rating_score, s.created_at
        "#;

        // Define the search snippet
        fn build_search_query<'a>(
            query_builder: &mut QueryBuilder<'a, sqlx::Postgres>,
            query_str: &'a str,
            threshold: f32,
        ) {
            query_builder.push(" ("); // Start of search query

            // Title search: (s.title ILIKE ... OR (s.title % ... AND similarity(...) >= ...))
            query_builder.push(" (s.title ILIKE '%' || ");
            query_builder.push_bind(query_str); // Binds value for ILIKE
            query_builder.push(" || '%' OR (s.title % ");
            query_builder.push_bind(query_str); // Binds value for trigram
            query_builder.push(" AND similarity(s.title,");
            query_builder.push_bind(query_str); // Binds value for similarity
            query_builder.push(") >= ");
            query_builder.push_bind(threshold); // Binds threshold
            query_builder.push("))");

            // Original title search: OR (s.original_title IS NOT NULL AND (...))
            query_builder.push(" OR (s.original_title IS NOT NULL AND (");
            query_builder.push(" s.original_title ILIKE '%' || ");
            query_builder.push_bind(query_str); // Binds value for ILIKE
            query_builder.push(" || '%'");
            query_builder.push(" OR (s.original_title % ");
            query_builder.push_bind(query_str); // Binds value for trigram
            query_builder.push(" AND similarity(s.original_title, ");
            query_builder.push_bind(query_str); // Binds value for similarity
            query_builder.push(") >= ");
            query_builder.push_bind(threshold); // Binds threshold
            query_builder.push(")");
            query_builder.push(" ))");

            query_builder.push(" )"); // End of search query
        }

        #[derive(Debug, FromRow)]
        struct QueryDefaultResult {
            id: i32,
            title: String,
            original_title: Option<String>,
            description: String,
            cover_image_url: String,
            last_chapter_found_in_storage: Option<f32>,
            updated_at: DateTime<Utc>,
            #[sqlx(json)]
            authors: serde_json::Value,
            #[sqlx(json)]
            categories: serde_json::Value,
            total_items: Option<i64>,
        }

        let mut query_builder = QueryBuilder::new("");
        let mut where_condition_added = false;

        // If any filter included, use the CTE
        if has_include_filters {
            query_builder
                .push("WITH filtered_series AS ( SELECT s.id FROM series s WHERE s.id IN (");

            // Subquery for include
            query_builder.push("SELECT series_id FROM series_categories WHERE category_id = ANY(");
            query_builder.push_bind(include_category_ids); // $1
            query_builder.push(") GROUP BY series_id HAVING COUNT(DISTINCT category_id) = ");
            query_builder.push_bind(include_category_ids.len() as i64); // $2
            query_builder.push(")"); // Close IN (...)

            // Exclude logic inside the CTE
            if has_exclude_filters {
                query_builder.push(" AND s.id NOT IN (SELECT series_id FROM series_categories WHERE category_id = ANY(");
                query_builder.push_bind(exclude_category_ids); // $3
                query_builder.push("))");
            }

            // Search logic inside the CTE for performance
            if let Some(query_str) = trimmed_search_query {
                query_builder.push(" AND ");

                build_search_query(&mut query_builder, query_str, SIMILARITY_THRESHOLD);
            }

            // Close CTE and build the main query
            query_builder.push(" ) ");
            query_builder.push(SELECT_FIELD);
            query_builder.push(" FROM filtered_series fs JOIN series s ON fs.id = s.id");
            query_builder.push(JOIN_LOGIC);
        } else {
            query_builder.push(SELECT_FIELD);
            query_builder.push(" FROM series s ");
            query_builder.push(JOIN_LOGIC);

            // Exclude logic in the WHERE clause
            if has_exclude_filters {
                query_builder.push(" WHERE NOT EXISTS (SELECT 1 FROM series_categories sc_exclude WHERE sc_exclude.series_id = s.id AND sc_exclude.category_id = ANY(");
                query_builder.push_bind(exclude_category_ids); // $1
                query_builder.push(")) ");

                where_condition_added = true;
            }

            // Search logic in the WHERE clause
            if let Some(query_str) = trimmed_search_query {
                if where_condition_added {
                    query_builder.push(" AND ");
                } else {
                    query_builder.push(" WHERE ");
                }

                build_search_query(&mut query_builder, query_str, SIMILARITY_THRESHOLD);

                // where_condition_added = true;
            }
        }

        // Final Assembly (Common to all logic paths)
        query_builder.push(GROUP_BY_LOGIC);

        // Add ORDER BY
        query_builder.push(" ORDER BY ");
        query_builder.push(order_by_clause);

        // Add LIMIT and OFFSET
        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit); // $... (last)
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset); // $... (very last)

        println!(
            "Executing SQL for Browse Series Paginated: {}",
            query_builder.sql()
        );

        // Execute build query
        let record_list = query_builder
            .build_query_as::<QueryDefaultResult>()
            .fetch_all(&self.pool)
            .await
            .context("Failed to dymanically build and execute series browse query")?;

        let total_items = record_list
            .first()
            .map_or(0, |row| row.total_items.unwrap_or(0));

        let series_list = record_list
            .into_iter()
            .map(|r| BrowseSeriesSearchResult {
                id: r.id,
                title: r.title,
                original_title: r.original_title,
                description: r.description,
                cover_image_url: r.cover_image_url,
                last_chapter_found_in_storage: r.last_chapter_found_in_storage,
                updated_at: r.updated_at,
                authors: r.authors,
                categories: r.categories,
            })
            .collect();

        Ok(PaginatedResult {
            items: series_list,
            total_items,
        })
    }

    // Paginated fetching of user search series
    pub async fn user_search_paginated_series(
        &self,
        search_query: &str,
    ) -> AnyhowResult<Vec<UserSearchPaginatedSeries>> {
        let trimmed_query = search_query.trim();

        if trimmed_query.is_empty() {
            return Ok(Vec::new());
        }

        const LIMIT: i64 = 25;
        const SIMILARITY_THRESHOLD: f32 = 0.20;

        let search_list = sqlx::query_as!(
            UserSearchPaginatedSeries,
            r#"
            SELECT
                s.id,
                s.title,
                s.original_title,
                s.cover_image_url,
                s.last_chapter_found_in_storage,
                s.updated_at,
                COALESCE(json_agg(DISTINCT a.name ORDER BY a.name) FILTER (WHERE a.id IS NOT NULL),
                            '[]'::json) as authors
            FROM series s
            LEFT JOIN series_authors sa ON s.id = sa.series_id
            LEFT JOIN authors a ON sa.author_id = a.id
            WHERE
                (
                    s.title ILIKE '%' || $1 || '%'
                    OR (s.title % $1 AND similarity(s.title, $1) >= $2)
                )
                OR
                (
                    s.original_title IS NOT NULL AND (
                        s.original_title ILIKE '%' || $1 || '%'
                        OR (s.original_title % $1 AND similarity(s.original_title, $1) >= $2)
                    )
                )
            GROUP BY s.id
            ORDER BY GREATEST(
                     similarity(s.title, $1),
                     similarity(COALESCE(s.original_title, ''), $1)
            ) DESC
            LIMIT $3
            "#,
            trimmed_query,
            SIMILARITY_THRESHOLD,
            LIMIT,
        )
        .fetch_all(&self.pool)
        .await
        .context("User failed to search series")?;

        Ok(search_list)
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
