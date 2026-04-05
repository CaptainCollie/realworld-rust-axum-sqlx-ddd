use crate::domain::{
    errors::AppError,
    models::{comment::Comment, profile::Profile},
    repositories::CommentRepository,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostgresCommentRepository {
    pool: PgPool,
}

impl PostgresCommentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommentRepository for PostgresCommentRepository {
    async fn add_comment(
        &self,
        slug: &str,
        author_id: Uuid,
        body: &str,
    ) -> Result<Comment, AppError> {
        let row = sqlx::query!(
            r#"
            WITH inserted_comment AS (
                INSERT INTO comments (body, author_id, article_id)
                VALUES ($1, $2, (SELECT id FROM articles WHERE slug = $3))
                RETURNING id, created_at, updated_at, body, author_id
            )
            SELECT 
                c.id, c.created_at, c.updated_at, c.body,
                u.username AS author_username, u.bio AS author_bio, u.image AS author_image,
                false AS "following!"
            FROM inserted_comment c
            JOIN users u ON u.id = c.author_id
            "#,
            body,
            author_id,
            slug
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Comment {
            id: row.id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            body: row.body,
            author: Profile {
                username: row.author_username,
                bio: row.author_bio,
                image: row.author_image,
                following: row.following,
            },
        })
    }

    async fn get_comments_by_article(
        &self,
        slug: &str,
        viewer_id: Option<Uuid>,
    ) -> Result<Vec<Comment>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                c.id, c.created_at, c.updated_at, c.body,
                u.username AS author_username, u.bio AS author_bio, u.image AS author_image,
                COALESCE(f.follower_id IS NOT NULL, false) AS "following!"
            FROM comments c
            JOIN articles a ON a.id = c.article_id
            JOIN users u ON u.id = c.author_id
            LEFT JOIN follows f ON f.followed_id = u.id AND f.follower_id = $1
            WHERE a.slug = $2
            ORDER BY c.created_at DESC
            "#,
            viewer_id,
            slug
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Comment {
                id: r.id,
                created_at: r.created_at,
                updated_at: r.updated_at,
                body: r.body,
                author: Profile {
                    username: r.author_username,
                    bio: r.author_bio,
                    image: r.author_image,
                    following: r.following,
                },
            })
            .collect())
    }

    async fn delete_comment(&self, comment_id: i32, author_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!(
            "DELETE FROM comments WHERE id = $1 AND author_id = $2",
            comment_id,
            author_id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            let exists = sqlx::query!("SELECT id FROM comments WHERE id = $1", comment_id)
                .fetch_optional(&self.pool)
                .await?
                .is_some();

            if exists {
                return Err(AppError::CommentForbidden);
            } else {
                return Err(AppError::CommentNotFound);
            }
        }
        Ok(())
    }

    async fn get_comment_author_id(&self, comment_id: i32) -> Result<Option<Uuid>, AppError> {
        let row = sqlx::query!("SELECT author_id FROM comments WHERE id = $1", comment_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.author_id))
    }
}
