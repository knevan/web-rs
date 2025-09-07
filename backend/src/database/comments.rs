use super::*;
use ammonia::Builder;
use pulldown_cmark::{Options, Parser, html};
use std::collections::{HashMap, HashSet};

impl DatabaseService {
    // Helper function to transform a flat list of comments into a nested tree structure.
    fn nested_comment_tree(&self, rows: Vec<CommentFlatRow>) -> Vec<Comment> {
        let mut comments_map: HashMap<i64, Comment> = HashMap::new();
        let mut root_comments: Vec<Comment> = Vec::new();

        for row in rows {
            let comment = Comment {
                id: row.id,
                parent_id: row.parent_id,
                content_html: row.content_html,
                created_at: row.created_at,
                updated_at: row.updated_at,
                user: CommentUser {
                    username: row.user_username,
                    avatar_url: row.user_avatar_url,
                },
                upvotes: row.upvotes,
                downvotes: row.downvotes,
                current_user_vote: row.current_user_vote,
                replies: Vec::new(),
            };
            comments_map.insert(comment.id, comment);
        }

        let comments_link_parent: Vec<Comment> =
            comments_map.values().cloned().collect();
        for comment in comments_link_parent {
            if let Some(parent_id) = comment.parent_id {
                if let Some(parent) = comments_map.get_mut(&parent_id) {
                    parent.replies.push(comment);
                }
            } else {
                root_comments.push(comment);
            }
        }
        root_comments
    }

    pub async fn get_comments(
        &self,
        entity_type: CommentEntityType,
        entity_id: i32,
        current_user_id: Option<i32>,
    ) -> AnyhowResult<Vec<Comment>> {
        let flat_comments: Vec<CommentFlatRow> = sqlx::query_as!(
            CommentFlatRow,
            r#"
            WITH RECURSIVE comment_thread AS (
                -- Anchor member: top-level comments
                SELECT * FROM comments
                WHERE comments_type = $1 AND comments_id = $2 AND parent_id IS NULL AND deleted_at IS NULL
                UNION ALL
                -- Recursive member: replies to comments already in the thread
                SELECT c.*
                FROM comments c
                JOIN comment_thread ct ON c.parent_id = ct.id
                WHERE c.deleted_at IS NULL
             ),
            vote_summary AS (
                SELECT
                    cv.comment_vote_id,
                    COUNT(*) FILTER (WHERE cv.vote_type = 1) AS upvotes,
                    COUNT(*) FILTER (WHERE cv.vote_type = -1) AS downvotes
                FROM comment_votes cv
                WHERE cv.comment_vote_id IN (SELECT id FROM comment_thread)
                GROUP BY cv.comment_vote_id
            )
            SELECT
                ct.id as "id!",
                ct.parent_id,
                ct.content_html as "content_html!",
                ct.created_at as "created_at!",
                ct.updated_at as "updated_at!",
                COALESCE(up.display_name, u.username) as "user_username!",
                up.avatar_url as "user_avatar_url",
                COALESCE(vs.upvotes, 0) as "upvotes!",
                COALESCE(vs.downvotes, 0) as "downvotes!",
                cv.vote_type as "current_user_vote: _"
            FROM comment_thread ct
            JOIN users u ON ct.user_id = u.id
            LEFT JOIN user_profiles up ON u.id = up.user_id
            LEFT JOIN vote_summary vs ON ct.id = vs.comment_vote_id
            LEFT JOIN comment_votes cv ON ct.id = cv.comment_vote_id AND cv.user_id = $3
            ORDER BY ct.created_at ASC
            "#,
            entity_type as _,
            entity_id,
            current_user_id
        )
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch comment thread")?;

        Ok(self.nested_comment_tree(flat_comments))
    }

    pub async fn create_new_comment(
        &self,
        user_id: i32,
        entity_type: CommentEntityType,
        entity_id: i32,
        content_markdown: &str,
        parent_id: Option<i64>,
    ) -> AnyhowResult<i64> {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);

        let parser = Parser::new_ext(content_markdown, options);

        let mut unsafe_html = String::new();
        html::push_html(&mut unsafe_html, parser);

        let content_html = Builder::new()
            .tags(HashSet::from([
                "p",
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
            ]))
            .link_rel(Some("nofollow noopener noreferrer"))
            .clean(&unsafe_html)
            .to_string();

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
            .fetch_one(&self.pool)
            .await
            .context("Failed to create new comments")?;

        Ok(new_comment_id)
    }

    pub async fn update_existing_comment(
        &self,
        comment_id: i64,
        user_id: i32,
        new_content_markdown: &str,
    ) -> AnyhowResult<Option<String>> {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);

        let parser = Parser::new_ext(new_content_markdown, options);
        let mut unsafe_html = String::new();
        html::push_html(&mut unsafe_html, parser);

        let new_content_html = Builder::new()
            .tags(HashSet::from([
                "p",
                "strong",
                "em",
                "a",
                "code",
                "pre",
                "blockquote",
            ]))
            .link_rel(Some("nofollow noopener noreferrer"))
            .clean(&unsafe_html)
            .to_string();

        let updated_html = sqlx::query_scalar!(
            r#"
            UPDATE comments
            SET
                content_user_markdown = $1,
                content_html = $2,
                updated_at = NOW()
            WHERE id = $3 AND user_id = $4
            RETURNING content_html
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
}
