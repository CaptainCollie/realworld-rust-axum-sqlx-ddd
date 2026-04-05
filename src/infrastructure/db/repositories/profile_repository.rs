use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{errors::AppError, models::profile::Profile, repositories::ProfileRepository};

pub struct PostgresProfileRepository {
    pool: PgPool,
}

impl PostgresProfileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProfileRepository for PostgresProfileRepository {
    async fn get_profile(
        &self,
        target_username: &str,
        viewer_id: Option<Uuid>,
    ) -> Result<Option<Profile>, AppError> {
        let row = sqlx::query!(
            r#"
                SELECT 
                    u.username, u.bio, u.image,
                    COALESCE(f.follower_id IS NOT NULL, false) AS "following!"
                FROM users u
                LEFT JOIN follows f ON f.followed_id = u.id AND f.follower_id = $1
                WHERE u.username = $2
                "#,
            viewer_id,
            target_username
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Profile {
            username: r.username,
            bio: r.bio,
            image: r.image,
            following: r.following,
        }))
    }

    async fn add_follow(&self, follower_id: Uuid, followed_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO follows (follower_id, followed_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
            follower_id,
            followed_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn remove_follow(&self, follower_id: Uuid, followed_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            DELETE FROM follows 
            WHERE follower_id = $1 AND followed_id = $2
            "#,
            follower_id,
            followed_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
