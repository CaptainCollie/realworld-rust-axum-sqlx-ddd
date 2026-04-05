use crate::domain::errors::AppError;
use crate::domain::models::user::{User, UserPasswordHash};
use crate::domain::repositories::UserRepository;
use async_trait::async_trait;
use sqlx::{PgPool, query};
use uuid::Uuid;

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, user: &User, password_hash: &UserPasswordHash) -> Result<(), AppError> {
        query!(
            r#"
            INSERT INTO users (id, username, email, bio, image, password_hash)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            user.id,
            user.username,
            user.email.to_string(),
            user.bio,
            user.image,
            password_hash.0
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error()
                && db_err.code() == Some("23505".into())
            {
                let constraint = db_err.constraint().unwrap_or_default();

                if constraint.contains("email") {
                    return AppError::Conflict {
                        field: "email".to_string(),
                        message: "has already been taken".to_string(),
                    };
                } else if constraint.contains("username") {
                    return AppError::Conflict {
                        field: "username".to_string(),
                        message: "has already been taken".to_string(),
                    };
                }
            }
            AppError::DatabaseError(e)
        })?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<(User, UserPasswordHash)>, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT id, username, email, bio, image, password_hash
            FROM users WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        let result = row.map(|r| {
            let user = User {
                id: r.id,
                username: r.username,
                email: r.email.parse().expect("Invalid email in database"),
                bio: r.bio,
                image: r.image,
            };
            let password_hash = UserPasswordHash(r.password_hash);
            (user, password_hash)
        });
        Ok(result)
    }

    async fn find_by_email(
        &self,
        email: &str,
    ) -> Result<Option<(User, UserPasswordHash)>, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT id, username, email, bio, image, password_hash 
            FROM users 
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        let result = row.map(|r| {
            let user = User {
                id: r.id,
                username: r.username,
                email: r.email.parse().expect("Invalid email in database"),
                bio: r.bio,
                image: r.image,
            };

            let password_hash = UserPasswordHash(r.password_hash);

            (user, password_hash)
        });

        Ok(result)
    }

    async fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Option<(User, UserPasswordHash)>, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT id, username, email, bio, image, password_hash
            FROM users WHERE username = $1
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await?;

        let result = row.map(|r| {
            let user = User {
                id: r.id,
                username: r.username,
                email: r.email.parse().expect("Invalid email in database"),
                bio: r.bio,
                image: r.image,
            };
            let password_hash = UserPasswordHash(r.password_hash);
            (user, password_hash)
        });
        Ok(result)
    }

    async fn update(&self, user: &User) -> Result<User, AppError> {
        let row = sqlx::query!(
            r#"
            UPDATE users 
            SET username = $2, email = $3, bio = $4, image = $5, updated_at = NOW()
            WHERE id = $1
            RETURNING id, username, email, bio, image
            "#,
            user.id,
            user.username,
            user.email.to_string(),
            user.bio,
            user.image
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::UserNotFound,
            _ => AppError::DatabaseError(e),
        })?;

        let updated_user = User {
            id: row.id,
            username: row.username,
            email: row.email.parse().expect("Invalid email in database"),
            bio: row.bio,
            image: row.image,
        };

        Ok(updated_user)
    }

    async fn update_password_hash(
        &self,
        user_id: Uuid,
        password_hash: &UserPasswordHash,
    ) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"
            UPDATE users
            SET password_hash = $2
            WHERE id = $1
            "#,
            user_id,
            password_hash.0,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::UserNotFound,
            _ => AppError::DatabaseError(e),
        })?;

        if result.rows_affected() == 0 {
            return Err(AppError::UserNotFound);
        }

        Ok(())
    }
}
