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
}
