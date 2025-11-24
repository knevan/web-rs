use std::collections::{HashMap, HashSet};

use ammonia::Builder;
use anyhow::anyhow;
use once_cell::sync::Lazy;
use pulldown_cmark::{html, Options, Parser};
use regex::Regex;

use super::*;

static SPOILER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\|\|(.*?)\|\|").expect("Regex pattern failed to compile (invalid)"));

impl DatabaseService {
    fn map_flat_row(row: CommentFlatRow) -> Comment {
        Comment {
            id: row.id,
            parent_id: row.parent_id,
            content_html: row.content_html,
            content_markdown: row.content_markdown,
            created_at: row.created_at,
            updated_at: row.updated_at,
            user: CommentUser {
                id: row.user_id,
                username: row.user_username,
                avatar_url: row.user_avatar_url,
            },
            upvotes: row.upvotes,
            downvotes: row.downvotes,
            current_user_vote: row.current_user_vote,
            replies: Vec::new(),
            attachment_urls: row.attachment_urls,
        }
    }

    // Helper function to transform a flat list of comments into a nested tree structure.
    fn nested_comment_tree(&self, rows: Vec<CommentFlatRow>, sort_by: CommentSort) -> Vec<Comment> {
        if rows.is_empty() {
            return Vec::new();
        }

        // A map to hold all child comments, grouped by their parent's ID for efficient lookup.
        let mut children_map: HashMap<i64, Vec<Comment>> = HashMap::with_capacity(rows.len());

        // A vector to store the root-level comments.
        let mut root_comments: Vec<Comment> = Vec::new();

        for row in rows {
            let comment = Self::map_flat_row(row);

            if let Some(parent_id) = comment.parent_id {
                // If it's a reply, add it to the children_map, keyed by its parent's ID.
                children_map.entry(parent_id).or_default().push(comment);
            } else {
                root_comments.push(comment);
            }
        }

        // Sort root comments
        match sort_by {
            CommentSort::Newest => {
                root_comments
                    .sort_unstable_by(|a, b| b.created_at.cmp(&a.created_at).then(b.id.cmp(&a.id)));
            }
            CommentSort::Oldest => {
                root_comments
                    .sort_unstable_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));
            }
            CommentSort::TopVote => root_comments.sort_unstable_by(|a, b| {
                let a_score = a.upvotes - a.downvotes;
                let b_score = b.upvotes - b.downvotes;
                b_score
                    .cmp(&a_score)
                    .then_with(|| b.created_at.cmp(&a.created_at))
            }),
        }

        let mut stack: Vec<&mut Comment> = root_comments.iter_mut().collect();

        while let Some(parent) = stack.pop() {
            if let Some(mut children) = children_map.remove(&parent.id) {
                // Sort by creation date
                children
                    .sort_unstable_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));

                parent.replies = children;

                // Push children to stack
                for child in &mut parent.replies {
                    stack.push(child);
                }
            }
        }

        root_comments
    }

    pub async fn get_comments(
        &self,
        entity_type: CommentEntityType,
        entity_id: i32,
        current_user_id: Option<i32>,
        sort_by: CommentSort,
    ) -> AnyhowResult<Vec<Comment>> {
        let flat_comments: Vec<CommentFlatRow> = sqlx::query_as!(
            CommentFlatRow,
            r#"
            WITH RECURSIVE comment_thread AS (
                -- Anchor member: top-level comments
                SELECT * FROM comments
                WHERE comments_type = $1 AND comments_id = $2 AND parent_id IS NULL
                UNION ALL
                -- Recursive member: replies to comments already in the thread
                SELECT c.*
                FROM comments c
                JOIN comment_thread ct ON c.parent_id = ct.id
            ),
            vote_summary AS (
                SELECT
                    cv.comment_vote_id,
                    COUNT(*) FILTER (WHERE cv.vote_type = 1) AS upvotes,
                    COUNT(*) FILTER (WHERE cv.vote_type = -1) AS downvotes
                FROM comment_votes cv
                WHERE cv.comment_vote_id IN (SELECT id FROM comment_thread)
                GROUP BY cv.comment_vote_id
            ),
            attachments_summary AS (
            -- Aggregate all attachment URLs for each comment into a JSON array
            SELECT
                comment_id,
                array_agg(file_url) as attachment_urls
            FROM comment_attachments
            WHERE comment_id IN (SELECT id FROM comment_thread)
            GROUP BY comment_id
        )
            SELECT
                ct.id as "id!",
                ct.parent_id,
                ct.content_html as "content_html!",
                ct.content_user_markdown as "content_markdown!",
                ct.created_at as "created_at!",
                ct.updated_at as "updated_at!",
                ct.user_id as "user_id!",
                COALESCE(up.display_name, u.username) as "user_username!",
                up.avatar_url as "user_avatar_url",
                COALESCE(vs.upvotes, 0) as "upvotes!",
                COALESCE(vs.downvotes, 0) as "downvotes!",
                cv.vote_type as "current_user_vote",
                ats.attachment_urls as "attachment_urls"
            FROM comment_thread ct
            JOIN users u ON ct.user_id = u.id
            LEFT JOIN user_profiles up ON u.id = up.user_id
            LEFT JOIN vote_summary vs ON ct.id = vs.comment_vote_id
            LEFT JOIN comment_votes cv ON ct.id = cv.comment_vote_id AND cv.user_id = $3
            LEFT JOIN attachments_summary ats ON ct.id = ats.comment_id
            -- ORDER BY ct.created_at ASC
            "#,
            entity_type as _,
            entity_id,
            current_user_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch comment thread")?;

        Ok(self.nested_comment_tree(flat_comments, sort_by))
    }

    fn process_comment_markdown(&self, markdown: &str) -> AnyhowResult<String> {
        // Process spoiler markdown ||spoiler|| to <span>
        let processed_spoiler_markdown =
            SPOILER_REGEX.replace_all(markdown, r#"<span class="spoiler-hook">$1</span>"#);

        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);

        let parser = Parser::new_ext(&processed_spoiler_markdown, options);

        let mut unsafe_html = String::new();
        html::push_html(&mut unsafe_html, parser);

        // Replace <strong> with <b> more efficient
        let unsafe_html = unsafe_html
            .replace("<strong>", "<b>")
            .replace("</strong>", "</b>");

        let allowed_tags = HashSet::from([
            "p",
            "b",
            "strong",
            "em",
            "a",
            "code",
            "pre",
            "blockquote",
            "ul",
            "ol",
            "li",
            "h1",
            "h2",
            "h3",
            "span",
        ]);

        let mut allowed_classes = HashMap::new();
        // Allow class "spoiler" for <span> tag
        allowed_classes.insert("span", HashSet::from(["spoiler-hook"]));

        let sanitized_html = Builder::new()
            .tags(allowed_tags)
            .add_tag_attributes("a", &["href"])
            .allowed_classes(allowed_classes)
            .link_rel(Some("nofollow noopener noreferrer"))
            .clean(&unsafe_html)
            .to_string();

        Ok(sanitized_html)
    }

    // Function to add new comment on existing comment tree list
    pub async fn get_comment_by_id(
        &self,
        comment_id: i64,
        current_user_id: Option<i32>,
    ) -> AnyhowResult<Option<Comment>> {
        let comment_row: Option<CommentFlatRow> = sqlx::query_as!(
            CommentFlatRow,
            r#"
            WITH vote_summary AS (
                SELECT
                    cv.comment_vote_id,
                    COUNT(*) FILTER (WHERE cv.vote_type = 1) AS upvotes,
                    COUNT(*) FILTER (WHERE cv.vote_type = -1) AS downvotes
                FROM comment_votes cv
                WHERE cv.comment_vote_id = $1
                GROUP BY cv.comment_vote_id
            ),
            attachments_summary AS (
            SELECT
                comment_id,
                array_agg(file_url) as attachment_urls
            FROM comment_attachments
            WHERE comment_id = $1
            GROUP BY comment_id
        )
            SELECT
                c.id as "id!",
                c.parent_id,
                c.content_html as "content_html!",
                c.content_user_markdown as "content_markdown!",
                c.created_at as "created_at!",
                c.updated_at as "updated_at!",
                c.user_id as "user_id!",
                COALESCE(up.display_name, u.username) as "user_username!",
                up.avatar_url as "user_avatar_url",
                COALESCE(vs.upvotes, 0) as "upvotes!",
                COALESCE(vs.downvotes, 0) as "downvotes!",
                cv.vote_type as "current_user_vote?",
                ats.attachment_urls as "attachment_urls?"
            FROM comments c
            JOIN users u ON c.user_id = u.id
            LEFT JOIN user_profiles up ON u.id = up.user_id
            LEFT JOIN vote_summary vs ON c.id = vs.comment_vote_id
            LEFT JOIN comment_votes cv ON c.id = cv.comment_vote_id AND cv.user_id = $2
            LEFT JOIN attachments_summary ats ON c.id = ats.comment_id
            WHERE c.id = $1 AND c.deleted_at IS NULL
            "#,
            comment_id,
            current_user_id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch comment by its id")?;

        Ok(comment_row.map(Self::map_flat_row))
    }

    pub async fn create_new_comment(
        &self,
        user_id: i32,
        entity_type: CommentEntityType,
        entity_id: i32,
        content_markdown: &str,
        parent_id: Option<i64>,
        attachment_keys: &[String],
    ) -> AnyhowResult<i64> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to start transaction")?;

        let content_html = self.process_comment_markdown(content_markdown)?;

        let new_comment_id = sqlx::query_scalar!(
            r#"
            INSERT INTO comments (user_id, comments_type, comments_id, parent_id, content_user_markdown, content_html)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
            user_id,
            entity_type as _,
            entity_id,
            parent_id,
            content_markdown,
            content_html
        )
            .fetch_one(&mut *tx)
            .await
            .context("Failed to create new comments")?;

        if !attachment_keys.is_empty() {
            for key in attachment_keys {
                sqlx::query!(
                    "INSERT INTO comment_attachments (comment_id, file_url) VALUES ($1, $2)",
                    new_comment_id,
                    key
                )
                .execute(&mut *tx)
                .await
                .context("Failed to insert comment attachment")?;
            }
        }

        tx.commit().await.context("Failed to commit transaction")?;

        Ok(new_comment_id)
    }

    pub async fn update_existing_comment(
        &self,
        comment_id: i64,
        user_id: i32,
        new_content_markdown: &str,
    ) -> AnyhowResult<Option<UpdateCommentResponse>> {
        let new_content_html = self.process_comment_markdown(new_content_markdown)?;

        let updated_html = sqlx::query_as!(
            UpdateCommentResponse,
            r#"
            UPDATE comments
            SET
                content_user_markdown = $1,
                content_html = $2,
                updated_at = NOW()
            WHERE id = $3 AND user_id = $4
            RETURNING id, content_user_markdown, content_html, updated_at
            "#,
            new_content_markdown,
            new_content_html,
            comment_id,
            user_id,
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to update existing comment")?;

        Ok(updated_html)
    }

    pub async fn delete_comment(
        &self,
        comment_id: i64,
        user_id: i32,
    ) -> AnyhowResult<DeleteCommentResult> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to start transaction")?;

        // Fetch all attachment keys associated with this comment *before* deletion.
        let attachment_object_key: Vec<String> = sqlx::query_scalar!(
            "SELECT file_url FROM comment_attachments WHERE comment_id = $1",
            comment_id
        )
        .fetch_all(&mut *tx)
        .await
        .context("Failed to fetch attachment keys")?;

        // Check if comment has replies
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

        let rows_affected: u64;

        if has_replies {
            let soft_delete_result = sqlx::query_as!(
                UpdateCommentResponse,
                r#"
                UPDATE comments
                SET
                    content_user_markdown = '',
                    content_html = '<p>[Deleted]</p>', -- Or any placeholder
                    deleted_at = NOW(),
                    updated_at = NOW()
                WHERE id = $1 AND user_id = $2
                RETURNING id, content_user_markdown, content_html, updated_at
                "#,
                comment_id,
                user_id
            )
            .fetch_optional(&mut *tx)
            .await
            .context("Failed to soft delete comment")?;

            if let Some(updated_comment) = soft_delete_result {
                // Since this is an UPDATE, ON DELETE CASCADE won't trigger
                // Manually delete the attachments
                if let Err(e) = sqlx::query!(
                    "DELETE FROM comment_attachments WHERE comment_id = $1",
                    comment_id
                )
                .execute(&mut *tx)
                .await
                {
                    tx.rollback().await.context("Failed to delete comment")?;
                    return Err(anyhow!(e).context("Failed to delete comment attachments"));
                }
                tx.commit().await.context("Failed to commit transaction")?;
                Ok(DeleteCommentResult::SoftDeleted(
                    updated_comment,
                    attachment_object_key,
                ))
            } else {
                tx.rollback().await.context("Failed to delete comment")?;
                Ok(DeleteCommentResult::NotFound)
            }
        } else {
            // No replies. We can safely delete the comment row.
            // Attachment will automatically deleted by `ON DELETE CASCADE`
            let hard_delete_result = sqlx::query!(
                r#"
                DELETE FROM comments
                WHERE id = $1 AND user_id = $2
                "#,
                comment_id,
                user_id
            )
            .execute(&mut *tx)
            .await
            .context("Failed to delete comment")?;

            rows_affected = hard_delete_result.rows_affected();

            if rows_affected == 0 {
                tx.rollback().await.context("Failed to delete comment")?;
                return Ok(DeleteCommentResult::NotFound);
            }

            tx.commit().await.context("Failed to commit transaction")?;

            Ok(DeleteCommentResult::HardDeleted(attachment_object_key))
        }
    }

    pub async fn vote_on_comment(
        &self,
        comment_id: i64,
        user_id: i32,
        vote_type: i16,
    ) -> AnyhowResult<CommentVoteResponse> {
        // Validate that vote_type is either 1 or -1
        if vote_type != 1 && vote_type != -1 {
            return Err(anyhow!("Invalid vote type"));
        }

        let mut tx = self.pool.begin().await?;

        let user_current_vote: Option<i16> = sqlx::query_scalar!(
            "SELECT vote_type FROM comment_votes WHERE comment_vote_id = $1 AND user_id = $2",
            comment_id,
            user_id
        )
        .fetch_optional(&mut *tx)
        .await
        .context("Failed to fetch comment vote")?;

        let mut final_user_vote: Option<i16> = Some(vote_type);

        // Decide db action base on vote status
        if Some(vote_type) == user_current_vote {
            // Delete vote if user clicking same button again
            sqlx::query!(
                "DELETE FROM comment_votes WHERE comment_vote_id = $1 AND user_id = $2",
                comment_id,
                user_id
            )
            .execute(&mut *tx)
            .await
            .context("Failed to delete comment")?;
            final_user_vote = None;
        } else {
            // New vote or changing vote
            sqlx::query!(
                r#"
                INSERT INTO comment_votes (comment_vote_id, user_id, vote_type) VALUES ($1, $2, $3)
                ON CONFLICT (comment_vote_id, user_id)
                DO UPDATE SET vote_type = EXCLUDED.vote_type
                "#,
                comment_id,
                user_id,
                vote_type
            )
            .execute(&mut *tx)
            .await
            .context("Failed to insert comment")?;
        }

        // Recalculate new total votes for comment
        let vote_counts = sqlx::query!(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE vote_type = 1) AS "upvotes!",
                COUNT(*) FILTER (WHERE vote_type = -1) AS "downvotes!"
            FROM comment_votes
            WHERE comment_vote_id = $1
            "#,
            comment_id
        )
        .fetch_one(&mut *tx)
        .await
        .context("Failed to recalculate total votes")?;

        tx.commit().await?;

        Ok(CommentVoteResponse {
            new_upvotes: vote_counts.upvotes,
            new_downvotes: vote_counts.downvotes,
            current_user_vote: final_user_vote,
        })
    }
}
