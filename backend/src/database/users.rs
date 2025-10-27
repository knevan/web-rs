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
    /// Fetch user by username or email
    pub async fn get_user_by_identifier(
        &self,
        identifier: &str,
    ) -> AnyhowResult<Option<Users>> {
        let user = sqlx::query_as!(
            Users,
                "SELECT id, username, email, password_hash, role_id FROM users WHERE email = $1 OR username = $1",
                identifier,
            ).fetch_optional(&self.pool).await.context("Failed to get user by identifier")?;
        Ok(user)
    }

    /// Create new user
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

    // Fetch user profiles data by id
    pub async fn get_user_profile_details(
        &self,
        user_id: i32,
    ) -> AnyhowResult<Option<UserProfileDetails>> {
        let profile = sqlx::query_as!(
            UserProfileDetails,
            r#"
            SELECT
                u.username,
                u.email,
                p.display_name,
                p.avatar_url
            FROM users u
            LEFT JOIN user_profiles p ON u.id = p.user_id
            WHERE u.id = $1
            "#,
            user_id
        )
            .fetch_optional(&self.pool)
            .await
            .context("Failed to get user profile details")?;

        Ok(profile)
    }

    // Update user display name and email
    pub async fn update_partial_user_profile(
        &self,
        user_id: i32,
        display_name: Option<String>,
        email: Option<String>,
    ) -> AnyhowResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to start transaction")?;

        if let Some(name) = display_name {
            sqlx::query!(
                r#"
                INSERT INTO user_profiles (user_id, display_name) VALUES ($1, $2)
                ON CONFLICT (user_id) DO UPDATE SET display_name = EXCLUDED.display_name
                "#,
                user_id,
                name
            )
                .execute(&mut *tx)
                .await
                .context("Failed to update user profile")?;
        }

        if let Some(mail) = email {
            sqlx::query!(
                "UPDATE users SET email = $1, updated_at = NOW() WHERE id = $2",
                mail,
                user_id
            )
                .execute(&mut *tx)
                .await
                .context("Failed to update user profile")?;
        }

        tx.commit().await.context("Failed to commit transaction")?;

        Ok(())
    }

    // Update user avatar
    pub async fn update_user_avatar(
        &self,
        user_id: i32,
        avatar_key: &str,
    ) -> AnyhowResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO user_profiles (user_id, avatar_url) VALUES ($1, $2)
            ON CONFLICT (user_id) DO UPDATE SET avatar_url = EXCLUDED.avatar_url
            "#,
            user_id,
            avatar_key
        )
            .execute(&self.pool)
            .await
            .context("Failed to update avatar user profile")?;

        Ok(())
    }

    // Updates user password
    pub async fn update_user_password(
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
            .context("Failed to update user profile")?;

        Ok(())
    }
    
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

        match search_query.filter(|q| !q.trim().is_empty()) {
            Some(search_match) => {
                let search_match = search_match.trim();
                let similarity_threshold = 0.20_f32;

                let records = sqlx::query!(
                    r#"
                    WITH base_search AS (
                        SELECT
                            u.id,
                            u.username,
                            u.email,
                            r.role_name,
                            -- Calculate similarity score for ranking
                            similarity(u.username || ' ' || u.email, $3) AS sim_score
                        FROM users u
                        JOIN roles r ON u.role_id = r.id
                        WHERE
                            -- ILIKE for substring matches
                            (u.username ILIKE '%' || $3 || '%')
                            OR
                            (u.email ILIKE '%' || $3 || '%')
                                -- fuzzy match trigram filtering
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
                                -- WHEN user_tsv @@ to_tsquery('simple', $4) THEN 8
                                ELSE 6
                            END as search_rank
                        FROM base_search
                     ),
                    total_count AS (
                        SELECT COUNT(*) AS total FROM ranked_results
                    )
                    SELECT
                        rr.id,
                        rr.username,
                        rr.email,
                        rr.role_name,
                        tc.total as total_items
                    FROM ranked_results rr
                    CROSS JOIN total_count tc
                    -- Order by the best rank, then by similarity, then by ID for stable sorting
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
                    .context("Failed to update user profile")?;

                let total_items = records
                    .first()
                    .map_or(0, |row| row.total_items.unwrap_or(0));

                let user_list = records
                    .into_iter()
                    .map(|row| UserWithRole {
                        id: row.id,
                        username: row.username,
                        email: row.email,
                        role_name: row.role_name,
                    })
                    .collect();

                Ok(PaginatedResult {
                    items: user_list,
                    total_items,
                })
            }
            None => {
                // No search - simple pagination
                let records = sqlx::query!(
                    r#"
                SELECT
                    u.id,
                    u.username,
                    u.email,
                    r.role_name,
                    COUNT(*) OVER() as total_items
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
                .context("Failed to get paginated users")?;

                let total_items = records
                    .first()
                    .map_or(0, |row| row.total_items.unwrap_or(0));

                let user_list = records
                    .into_iter()
                    .map(|row| UserWithRole {
                        id: row.id,
                        username: row.username,
                        email: row.email,
                        role_name: row.role_name,
                    })
                    .collect();

                Ok(PaginatedResult {
                    items: user_list,
                    total_items,
                })
            }
        }
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

    // Update password hash for a given user ID
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

}
