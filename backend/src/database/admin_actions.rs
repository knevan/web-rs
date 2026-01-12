use super::*;
use crate::api::admin::ReportView;
use crate::api::extractor::Role;

// =========================================================================
// Admin Series Management
// =========================================================================
impl DatabaseService {
    pub async fn add_new_series(&self, data: &NewSeriesData<'_>) -> AnyhowResult<i32> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to begin transaction")?;

        let host = get_host_from_url(Some(data.source_url));

        let new_series_id = sqlx::query_scalar!(
            r#"
            INSERT INTO series
            (title, original_title, description, cover_image_url, current_source_url, source_website_host, check_interval_minutes)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
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
            .context("Failed to add series with sqlx")?;

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
                ).execute(&mut *tx).await.context(format!("Failed to link author {} to ", name))?;
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
                    .context(format!("Failed to link category {} to series", category_id))?;
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
        .context("Failed to update series with sqlx")?;

        if let Some(author_names) = data.authors {
            sqlx::query!("DELETE FROM series_authors WHERE series_id = $1", series_id)
                .execute(&mut *tx)
                .await
                .context("Failed to delete existing authors for series")?;

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
                .context(format!("Failed to find or create author: {}", name))?;

                sqlx::query!(
                    "INSERT INTO series_authors (series_id, author_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                    series_id,
                    author_id
                )
                    .execute(&mut *tx)
                    .await
                    .context(format!("Failed to link author {} to series", name))?;
            }
        }

        if let Some(category_ids) = data.category_ids {
            sqlx::query!(
                "DELETE FROM series_categories WHERE series_id = $1",
                series_id
            )
            .execute(&mut *tx)
            .await
            .context("Failed to delete existing categories for series")?;

            if !category_ids.is_empty() {
                for category_id in category_ids {
                    sqlx::query!(
                        "INSERT INTO series_categories (series_id, category_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                        series_id,
                        category_id
                    )
                        .execute(&mut *tx)
                        .await
                        .context(format!("Failed to link category {} to series", category_id))?;
                }
            }
        }

        tx.commit().await.context("Failed to commit transaction")?;

        Ok(result.rows_affected())
    }

    // Get paginated series search list for admin panel
    pub async fn get_admin_paginated_series(
        &self,
        page: u32,
        page_size: u32,
        search_query: Option<&str>,
    ) -> AnyhowResult<PaginatedResult<SeriesWithAuthors>> {
        let page = page.max(1);
        let limit = page_size as i64;
        let offset = (page as i64 - 1) * limit;

        #[derive(Debug, FromRow)]
        struct QueryResult {
            id: i32,
            title: String,
            original_title: Option<String>,
            description: String,
            cover_image_url: String,
            current_source_url: String,
            updated_at: DateTime<Utc>,
            processing_status: SeriesStatus,
            #[sqlx(json)]
            authors: serde_json::Value,
            total_items: Option<i64>,
        }

        let record_list = match search_query.filter(|q| !q.trim().is_empty()) {
            Some(search_match) => {
                let search_match = search_match.trim();
                let similarity_threshold = 0.20_f32;

                sqlx::query_as!(
                    QueryResult,
                    r#"
                    WITH base_search AS (
                    SELECT
                        s.id, s.title, s.original_title, s.description, s.cover_image_url,
                        s.current_source_url, s.updated_at, s.processing_status,
                        -- Calculate similarity score for ranking
                        similarity(s.title, $3) as sim_score
                    FROM series s
                    WHERE
                        s.title ILIKE '%' || $3 || '%'
                    OR
                        (s.title % $3 AND similarity(s.title, $3) >= $4)
                    ),
                    ranked_results AS (
                        SELECT
                            *,
                            CASE
                                WHEN title ILIKE $3 THEN 10
                                WHEN title ILIKE $3 || '%' THEN 8
                                WHEN title ILIKE '%' || $3 || '%' THEN 6
                                ELSE 4
                            END as search_rank
                        FROM base_search
                    ),
                    total_count AS (
                        SELECT COUNT(*) AS total FROM ranked_results
                    )
                    SELECT
                        rr.id, rr.title, rr.original_title, rr.description,
                        rr.cover_image_url, rr.current_source_url, rr.updated_at,
                        rr.processing_status as "processing_status: SeriesStatus",
                        -- Aggregate author names into a JSON array for each series
                        COALESCE(
                            json_agg(a.name) FILTER (WHERE a.id IS NOT NULL),
                            '[]'::json
                        ) AS "authors!",
                        tc.total as total_items
                    FROM ranked_results rr
                    CROSS JOIN total_count tc
                    LEFT JOIN series_authors sa ON rr.id = sa.series_id
                    LEFT JOIN authors a ON sa.author_id = a.id
                    GROUP BY
                        rr.id, rr.title, rr.original_title, rr.description, rr.cover_image_url,
                        rr.current_source_url, rr.updated_at, rr.processing_status,
                        rr.search_rank, rr.sim_score, tc.total
                    -- Order by the best rank, then by similarity, then by ID for stable sorting
                    ORDER BY rr.search_rank DESC, rr.sim_score DESC, rr.id ASC
                    LIMIT $1
                    OFFSET $2
                    "#,
                    limit,
                    offset,
                    search_match,
                    similarity_threshold,
                )
                .fetch_all(&self.pool)
                .await
                .context("Failed to query all series")
            }
            None => {
                // No search - simple pagination
                sqlx::query_as!(
                    QueryResult,
                    r#"
                    SELECT
                        s.id, s.title, s.original_title, s.description, s.cover_image_url,
                        s.current_source_url, s.updated_at,
                        s.processing_status as "processing_status: SeriesStatus",
                        COALESCE(
                            json_agg(a.name) FILTER (WHERE a.id IS NOT NULL),
                            '[]'::json
                        ) as "authors!",
                        COUNT(*) OVER () as total_items
                    FROM
                        series s
                    LEFT JOIN series_authors sa ON s.id = sa.series_id
                    LEFT JOIN authors a ON sa.author_id = a.id
                    GROUP BY s.id
                    ORDER BY s.updated_at DESC
                    LIMIT $1 OFFSET $2
                    "#,
                    limit,
                    offset
                )
                .fetch_all(&self.pool)
                .await
                .context("Failed to get paginated series without search")
            }
        }?;

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

    pub async fn mark_series_for_deletion(&self, series_id: i32) -> AnyhowResult<u64> {
        let result = sqlx::query!(
            "UPDATE series
            SET processing_status = $1, updated_at = NOW()
            WHERE id = $2
            AND processing_status NOT IN ($3, $4)",
            SeriesStatus::PendingDeletion as _,
            series_id,
            SeriesStatus::PendingDeletion as _,
            SeriesStatus::Deleting as _,
        )
        .execute(&self.pool)
        .await
        .context("Failed to mark series for deletion with sqlx")?;

        Ok(result.rows_affected())
    }

    pub async fn delete_series_by_id(&self, series_id: i32) -> AnyhowResult<u64> {
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
        sqlx::query!("DELETE FROM series_authors WHERE series_id = $1", series_id)
            .execute(&mut *tx)
            .await
            .context("Failed to delete series-authors links")?;

        let result = sqlx::query!("DELETE FROM series WHERE id = $1", series_id)
            .execute(&mut *tx)
            .await
            .context("Failed to delete series")?;

        tx.commit()
            .await
            .context("Failed to commit transaction for series deletion")?;

        Ok(result.rows_affected())
    }

    pub async fn create_category_tag(&self, name: &str) -> AnyhowResult<CategoryTag> {
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
}

// =========================================================================
// Admin Chapter Management
// =========================================================================
impl DatabaseService {
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
        .fetch_optional(&mut *tx)
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

            let result = sqlx::query!("DELETE FROM series_chapters WHERE id = $1", chapter_id)
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
}

// =========================================================================
// Admin User Management
// =========================================================================
impl DatabaseService {
    // Get paginated user search list for admin panel
    pub async fn get_admin_paginated_user(
        &self,
        page: u32,
        page_size: u32,
        search_query: Option<&str>,
    ) -> AnyhowResult<PaginatedResult<UserWithRole>> {
        let page = page.max(1);
        let limit = page_size as i64;
        let offset = (page as i64 - 1) * limit;

        struct UserRow {
            id: i32,
            username: String,
            email: String,
            role_id: i32,
            role_name: String,
            is_active: Option<bool>,
            total_items: Option<i64>,
        }

        let records: Vec<UserRow> = match search_query.filter(|q| !q.trim().is_empty()) {
            Some(search_match) => {
                let search_match = search_match.trim();
                let similarity_threshold = 0.30_f32;

                sqlx::query_as!(
                    UserRow,
                    r#"
                    WITH search_results AS (
                        SELECT
                            u.id,
                            u.username,
                            u.email,
                            u.role_id,
                            u.is_active,
                            r.role_name,
                            similarity(u.username || ' ' || u.email, $3) AS sim_score
                        FROM users u
                        JOIN roles r ON u.role_id = r.id
                        WHERE
                            (u.username ILIKE '%' || $3 || '%')
                            OR
                            (u.email ILIKE '%' || $3 || '%')
                            OR
                            (
                                (u.username || ' ' || u.email) % $3
                                AND
                                similarity(u.username || ' ' || u.email, $3) >= $4
                            )
                    ),
                    ranked_results AS (
                        SELECT
                            *,
                            CASE
                                WHEN username ILIKE $3 OR email ILIKE $3 THEN 10
                                WHEN username ILIKE '%' || $3 || '%' OR email ILIKE '%' || $3 || '%' THEN 8
                                ELSE 6
                            END as search_rank
                            -- Removed redundant sim_score definition here to fix 'ambiguous' error
                        FROM search_results
                     ),
                    total_count AS (
                        SELECT COUNT(*) AS total FROM ranked_results
                    )
                    SELECT
                        rr.id,
                        rr.username,
                        rr.email,
                        rr.role_name,
                        rr.role_id,
                        rr.is_active,
                        tc.total as total_items
                    FROM ranked_results rr
                    CROSS JOIN total_count tc
                    ORDER BY rr.search_rank DESC, rr.sim_score DESC, rr.id ASC
                    LIMIT $1 
                    OFFSET $2
                    "#,
                    limit,
                    offset,
                    search_match,
                    similarity_threshold
                )
                    .fetch_all(&self.pool)
                    .await
                    .context("Failed to search paginated users")?
            }
            None => sqlx::query_as!(
                UserRow,
                r#"
                    SELECT
                        u.id,
                        u.username,
                        u.email,
                        u.role_id,
                        r.role_name,
                        u.is_active,
                        COUNT(*) OVER() as "total_items"
                    FROM users u
                    JOIN roles r ON u.role_id = r.id
                    ORDER BY u.id ASC
                    LIMIT $1 OFFSET $2
                    "#,
                limit,
                offset
            )
            .fetch_all(&self.pool)
            .await
            .context("Failed to get paginated users")?,
        };

        let total_items = records.first().and_then(|row| row.total_items).unwrap_or(0);

        let user_list = records
            .into_iter()
            .map(|row| UserWithRole {
                id: row.id,
                username: row.username,
                email: row.email,
                role_name: row.role_name,
                role_id: row.role_id,
                is_active: row.is_active.unwrap_or(false),
            })
            .collect();

        Ok(PaginatedResult {
            items: user_list,
            total_items,
        })
    }

    // Delete user by ID (admin)
    pub async fn admin_delete_user(&self, user_id: i32) -> AnyhowResult<u64> {
        let result = sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
            .execute(&self.pool)
            .await
            .context("Failed to delete user")?;

        Ok(result.rows_affected())
    }

    /// Partial Update user details (admin)
    /// This function updates only the provided fields using a "Fetch-Merge-Update" pattern
    /// It returns the updated user data or None if the user was not found
    pub async fn admin_update_user(
        &self,
        user_id: i32,
        username: Option<&str>,
        email: Option<&str>,
        role_id: Option<i32>,
        is_active: Option<bool>,
        actor_role: Role,
    ) -> AnyhowResult<Option<UserWithRole>> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to begin transaction")?;

        // Fetch the current user data (and lock the row for update)
        let current_user = sqlx::query!(
            "SELECT u.username, u.email, u.role_id, u.is_active, r.role_name
            FROM users u
            JOIN roles r ON u.role_id = r.id
            WHERE u.id = $1
            FOR UPDATE",
            user_id
        )
        .fetch_optional(&mut *tx)
        .await
        .context("Failed to fetch user")?;

        // If user doesnt exist, rollback and return None
        let Some(current_user) = current_user else {
            tx.rollback().await.context("Failed to rollback user")?;
            return Ok(None);
        };

        // Get target role enum
        let target_role = Role::from_name(&current_user.role_name).unwrap_or(Role::User);

        // Check if actor has permission to modify target user
        // Admin (2) CANT change SuperAdmin (3) -> 2 <= 3 (Failed)
        // Admin (2) CANT change Admin (2) -> 2 <= 2 (Failed)
        // Admin (2) CAN change Moderator (1) -> 2 <= 1 (Pass)
        if actor_role <= target_role {
            tx.rollback().await.context("Failed to rollback user")?;
            anyhow::bail!(
                "FORBIDDEN: You do not have permission to modify a user with an equal or higher role."
            );
        }

        let mut new_role_id = current_user.role_id;

        if let Some(req_role_id) = role_id
            && req_role_id != current_user.role_id
        {
            let new_role_name =
                sqlx::query_scalar!("SELECT role_name FROM roles WHERE id = $1", new_role_id)
                    .fetch_optional(&mut *tx)
                    .await
                    .context("Failed to fetch role_id")?
                    .ok_or_else(|| anyhow::anyhow!("Invalid role_id: {}", new_role_id))?;

            let new_role_enum = Role::from_name(&new_role_name).unwrap_or(Role::User);

            if new_role_enum >= actor_role {
                tx.rollback().await.context("Failed to rollback user")?;
                anyhow::bail!("FORBIDDEN: You cannot assign a role higher than your own.");
            }
            new_role_id = req_role_id;
        }

        // Merge: Use new value if Some, otherwise keep the current value
        let new_username = username.unwrap_or(&current_user.username);
        let new_email = email.unwrap_or(&current_user.email);
        let new_is_active = is_active.or(current_user.is_active);

        // Check for conflicts (username or email) with *other* users
        // Only check if username or email is actually changing
        if (username.is_some() && username != Some(&current_user.username))
            || (email.is_some() && email != Some(&current_user.email))
        {
            let conflict = sqlx::query_scalar!(
                "SELECT 1 FROM users WHERE (username = $1 OR email = $2) AND id != $3 LIMIT 1",
                new_username,
                new_email,
                user_id
            )
            .fetch_optional(&mut *tx)
            .await
            .context("Failed to check for username/email conflict")?;

            if conflict.is_some() {
                tx.rollback().await.context("Failed to rollback user")?;
                // Return a specific error message that the handler can catch
                anyhow::bail!(
                    "Username or email already exists for another user with id {}",
                    user_id
                );
            }
        }

        // Update the user with merged data
        sqlx::query!(
            r#"
            UPDATE users
            SET username = $1, email = $2, role_id = $3, is_active = $4, updated_at = NOW()
            WHERE id = $5
            "#,
            new_username,
            new_email,
            new_role_id,
            new_is_active,
            user_id
        )
        .execute(&mut *tx)
        .await
        .context("Failed to update user")?;

        // Fetch the updated user data to return
        let updated_user = sqlx::query_as!(
            UserWithRole,
            r#"
            SELECT
                u.id,
                u.username,
                u.email,
                u.role_id,
                COALESCE(u.is_active, false) as "is_active!",
                r.role_name
            FROM users u
            JOIN roles r ON u.role_id = r.id
            WHERE u.id = $1
            "#,
            user_id
        )
        .fetch_one(&mut *tx)
        .await
        .context("Failed to fetch updated user")?;

        // Commit the transaction
        tx.commit().await.context("Failed to commit transaction")?;

        Ok(Some(updated_user))
    }
}

// =========================================================================
// Admin Comment Management
// =========================================================================
impl DatabaseService {
    // Delete comment as admin
    pub async fn admin_delete_comment(
        &self,
        comment_id: i64,
        requestor_role_id: i32,
    ) -> AnyhowResult<DeleteCommentResult> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to begin transaction")?;

        let target_info = sqlx::query!(
            r#"
            SELECT
                u.role_id,
                c.user_id
            FROM comments c
            JOIN users u ON c.user_id = u.id
            WHERE c.id = $1
            "#,
            comment_id
        )
        .fetch_optional(&mut *tx)
        .await
        .context("Failed to fetch comment info")?;

        let target_user_role_id = match target_info {
            Some(record) => record.role_id,
            None => {
                return Ok(DeleteCommentResult::NotFound);
            }
        };

        println!("DEBUG: Requestor Role ID: {}", requestor_role_id);
        println!("DEBUG: Target User Role ID: {}", target_user_role_id);

        // Validation Tiered logic
        // Role: SuperAdmin=1, Admin=2, Moderator=3, User=4
        let is_super_admin = requestor_role_id == 1;

        // If not super admin, check hierarchy.
        // We deny if requestor_role_id is Greater or Equal to target_user_role_id.
        if !is_super_admin && requestor_role_id >= target_user_role_id {
            tx.rollback().await?;
            return Ok(DeleteCommentResult::InsufficientPermissions);
        }

        let attachment_object_key: Vec<String> = sqlx::query_scalar!(
            "SELECT file_url FROM comment_attachments WHERE comment_id = $1",
            comment_id
        )
        .fetch_all(&mut *tx)
        .await
        .context("Failed to fetch attachment keys")?;

        let has_replies: bool = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM comments WHERE parent_id = $1 AND deleted_at IS NULL
            )
            "#,
            comment_id
        )
        .fetch_one(&mut *tx)
        .await
        .context("Failed to check for replies")?
        .context("EXISTS query returned NULL, which should not happen")?;

        let row_affected: u64;

        if has_replies {
            let soft_delete_result = sqlx::query_as!(
                UpdateCommentResponse,
                r#"
                UPDATE comments
                SET
                    content_user_markdown = '',
                    content_html = '<p>[Removed by Mod]</p>',
                    deleted_at = NOW(),
                    updated_at = NOW()
                WHERE id = $1
                RETURNING id, content_user_markdown, content_html, updated_at, (deleted_at IS NOT NULL) as "is_deleted!"
                "#,
                comment_id
            )
                .fetch_optional(&mut *tx)
                .await
                .context("Failed to soft delete comment")?;

            if let Some(updated_comment) = soft_delete_result {
                sqlx::query!(
                    "DELETE FROM comment_attachments WHERE comment_id = $1",
                    comment_id
                )
                .execute(&mut *tx)
                .await
                .context("Failed to delete comment attachments")?;

                tx.commit().await.context("Failed to comment deletion")?;

                Ok(DeleteCommentResult::SoftDeleted(
                    updated_comment,
                    attachment_object_key,
                ))
            } else {
                tx.rollback().await.context("Failed to comment deletion")?;
                Ok(DeleteCommentResult::NotFound)
            }
        } else {
            let hard_delete_result = sqlx::query!("DELETE FROM comments WHERE id = $1", comment_id)
                .execute(&mut *tx)
                .await
                .context("Failed to delete comment")?;

            row_affected = hard_delete_result.rows_affected();

            if row_affected == 0 {
                tx.rollback().await.context("Failed to delete comment")?;
                return Ok(DeleteCommentResult::NotFound);
            }

            tx.commit().await.context("Failed to commit transaction")?;

            Ok(DeleteCommentResult::HardDeleted(attachment_object_key))
        }
    }
}

// =========================================================================
// Admin Reports Management
// =========================================================================
impl DatabaseService {
    pub async fn get_admin_paginated_pending_reports(
        &self,
        page: u32,
        page_size: u32,
        search_query: Option<&str>,
    ) -> AnyhowResult<PaginatedResult<ReportView>> {
        let limit = page_size as i64;
        let offset = (page.max(1) as i64 - 1) * limit;

        #[derive(Debug, FromRow)]
        struct RawReportRow {
            id: i32,
            reporter_username: String,
            reporter_id: i32,
            created_at: DateTime<Utc>,
            reason: ReportReason,
            chapter_id: Option<i32>,
            chapter_number: Option<f32>,
            chapter_series_title: Option<String>,
            comment_id: Option<i64>,
            comment_preview: Option<String>,
            total_items: Option<i64>,
        }

        let records = match search_query.filter(|q| !q.trim().is_empty()) {
            Some(search_match) => {
                let search_match = search_match.trim();
                let similarity_threshold = 0.20_f32;

                sqlx::query_as!(
                    RawReportRow,
                    r#"
                    WITH search_candidates AS (
                        SELECT
                            r.id,
                            u.username as reporter_username,
                            r.reporter_id,
                            r.created_at,
                            r.reason,
                            sc.id as chapter_id,
                            sc.chapter_number,
                            s.title as chapter_series_title,
                            c.id as comment_id,
                            c.content_user_markdown,
                            c.content_html,
                            GREATEST(
                                similarity(u.username, $3),
                                similarity(COALESCE(s.title, ''), $3),
                                similarity(COALESCE(c.content_user_markdown, ''), $3)
                            ) as sim_score
                        FROM reports r
                        INNER JOIN users u ON r.reporter_id = u.id
                        LEFT JOIN series_chapters sc ON r.chapter_id = sc.id
                        LEFT JOIN series s ON sc.series_id = s.id
                        LEFT JOIN comments c ON r.comment_id = c.id
                        WHERE
                            -- Reporter Username
                            (u.username ILIKE '%' || $3 || '%' OR (u.username % $3 AND similarity(u.username, $3) >= $4))
                            OR
                            -- Series Title (only if related to series)
                            (s.title IS NOT NULL AND (s.title ILIKE '%' || $3 || '%' OR (s.title % $3 AND similarity(s.title, $3) >= $4)))
                            OR
                            -- Comment Content (only if related to comment)
                            (c.content_user_markdown IS NOT NULL AND c.content_user_markdown ILIKE '%' || $3 || '%')
                    ),
                    ranked_results AS (
                        SELECT *,
                            CASE
                                -- Reporter Username
                                WHEN reporter_username ILIKE $3 THEN 10
                                WHEN reporter_username ILIKE $3 || '%' THEN 9
                                -- Series Title
                                WHEN chapter_series_title ILIKE '%' || $3 || '%' THEN 8
                                -- Comment Content
                                ELSE 5
                            END as search_rank
                        FROM search_candidates
                    ),
                    total_count AS (
                        SELECT COUNT(*) as total FROM ranked_results
                    )
                    SELECT
                        rr.id,
                        rr.reporter_username,
                        rr.reporter_id,
                        rr.created_at,
                        rr.reason as "reason: ReportReason",
                        rr.chapter_id,
                        rr.chapter_number,
                        rr.chapter_series_title,
                        rr.comment_id,
                        SUBSTRING(rr.content_html, 1, 50) as comment_preview,
                        tc.total as total_items
                    FROM ranked_results rr
                    CROSS JOIN total_count tc
                    ORDER BY rr.search_rank DESC, rr.sim_score DESC, rr.created_at DESC
                    LIMIT $1
                    OFFSET $2
                    "#,
                    limit,
                    offset,
                    search_match,
                    similarity_threshold,
                )
                    .fetch_all(&self.pool)
                    .await
                    .context("Failed to fetch reports with sqlx")?
            }
            None => sqlx::query_as!(
                RawReportRow,
                r#"
                    SELECT
                        r.id,
                        u.username as reporter_username,
                        r.reporter_id,
                        r.created_at,
                        r.reason as "reason: ReportReason",
                        sc.id as chapter_id,
                        sc.chapter_number as chapter_number,
                        s.title as chapter_series_title,
                        c.id as comment_id,
                        SUBSTRING(c.content_html, 1, 50) as comment_preview,
                        COUNT(*) OVER() as total_items
                    FROM reports r
                    INNER JOIN users u ON r.reporter_id = u.id
                    LEFT JOIN series_chapters sc ON r.chapter_id = sc.id
                    LEFT JOIN series s ON sc.series_id = s.id
                    LEFT JOIN comments c ON r.comment_id = c.id
                    ORDER BY r.created_at DESC
                    LIMIT $1
                    OFFSET $2
                    "#,
                limit,
                offset,
            )
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch all pending reports")?,
        };

        let total_items = records
            .first()
            .map_or(0, |row| row.total_items.unwrap_or(0));

        let items: Vec<ReportView> = records
            .into_iter()
            .map(|row| ReportView {
                id: row.id,
                reporter_username: row.reporter_username,
                reporter_id: row.reporter_id,
                created_at: row.created_at,
                reason: row.reason,
                chapter_id: row.chapter_id,
                chapter_number: row.chapter_number,
                chapter_series_title: row.chapter_series_title,
                comment_id: row.comment_id,
                comment_preview: row.comment_preview,
            })
            .collect();

        Ok(PaginatedResult { items, total_items })
    }

    pub async fn admin_resolve_reports(&self, report_id: i32) -> AnyhowResult<()> {
        sqlx::query!("DELETE FROM reports WHERE id = $1", report_id)
            .execute(&self.pool)
            .await
            .context("Failed to delete reports")?;

        Ok(())
    }
}
