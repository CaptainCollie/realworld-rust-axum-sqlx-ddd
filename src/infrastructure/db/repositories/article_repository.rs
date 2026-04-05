use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::domain::{
    errors::AppError,
    models::{
        article::{Article, ArticleFilter},
        profile::Profile,
    },
    repositories::ArticleRepository,
};

#[derive(FromRow)]
struct ArticleRow {
    id: uuid::Uuid,
    slug: String,
    title: String,
    description: String,
    body: String,
    tag_list: Vec<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    author_username: String,
    author_bio: Option<String>,
    author_image: Option<String>,
    following: bool,
    favorited: bool,
    favorites_count: i64,
    total_count: i64,
    author_id: Uuid,
}

pub struct PostgresArticleRepository {
    pool: sqlx::PgPool,
}

impl PostgresArticleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ArticleRepository for PostgresArticleRepository {
    async fn create(
        &self,
        slug: &str,
        title: &str,
        description: &str,
        body: &str,
        author_id: Uuid,
        tags: &[String],
    ) -> Result<Article, AppError> {
        let mut tx = self.pool.begin().await?;

        let article_id = sqlx::query!(
            r#"
        INSERT INTO articles (slug, title, description, body, author_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
            slug,
            title,
            description,
            body,
            author_id
        )
        .fetch_one(&mut *tx)
        .await?;

        for tag_name in tags {
            let tag_id = sqlx::query!(
                r#"
            INSERT INTO tags (name) VALUES ($1)
            ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
            RETURNING id
            "#,
                tag_name
            )
            .fetch_one(&mut *tx)
            .await?
            .id;

            sqlx::query!(
                "INSERT INTO article_tags (article_id, tag_id) VALUES ($1, $2)",
                article_id.id,
                tag_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        self.get_by_slug(slug, None)
            .await?
            .ok_or(AppError::Internal("Failed to fetch created article".into()))
    }

    async fn get_by_slug(
        &self,
        slug: &str,
        viewer_id: Option<Uuid>,
    ) -> Result<Option<Article>, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT 
                a.id, a.author_id, a.slug, a.title, a.description, a.body, 
                a.created_at, a.updated_at,
                u.username AS author_username, 
                u.bio AS author_bio, 
                u.image AS author_image,
                COALESCE(f.follower_id IS NOT NULL, false) AS "following!",
                COALESCE((SELECT COUNT(*) FROM favorites WHERE article_id = a.id), 0) AS "favorites_count!",
                EXISTS(SELECT 1 FROM favorites WHERE article_id = a.id AND user_id = $1) AS "favorited!",
                COALESCE(ARRAY_AGG(t.name) FILTER (WHERE t.name IS NOT NULL), '{}') AS "tag_list!"
            FROM articles a
            INNER JOIN users u ON a.author_id = u.id
            LEFT JOIN follows f ON f.followed_id = u.id AND f.follower_id = $1
            LEFT JOIN article_tags at ON at.article_id = a.id
            LEFT JOIN tags t ON t.id = at.tag_id
            WHERE a.slug = $2
            GROUP BY a.id, u.id, f.follower_id
            "#,
            viewer_id,
            slug
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Article {
            id: r.id,
            slug: r.slug,
            title: r.title,
            description: r.description,
            body: r.body,
            tag_list: r.tag_list,
            created_at: r.created_at,
            updated_at: r.updated_at,
            favorited: r.favorited,
            favorites_count: r.favorites_count as u32,
            author_id: r.author_id,
            author: Profile {
                username: r.author_username,
                bio: r.author_bio,
                image: r.author_image,
                following: r.following,
            },
        }))
    }

    async fn delete(&self, slug: &str, author_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!(
            "DELETE FROM articles WHERE slug = $1 AND author_id = $2",
            slug,
            author_id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::ArticleNotFound);
        }

        Ok(())
    }

    async fn list_articles(
        &self,
        filter: ArticleFilter,
        viewer_id: Option<Uuid>,
    ) -> Result<(Vec<Article>, i64), AppError> {
        let mut query_builder: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
            r#"
            SELECT 
                a.id, a.author_id, a.slug, a.title, a.description, a.body, 
                a.created_at, a.updated_at,
                u.username AS author_username, u.bio AS author_bio, u.image AS author_image,
                COALESCE(f.follower_id IS NOT NULL, false) AS following,
                COUNT(*) OVER() AS total_count, 
                EXISTS(SELECT 1 FROM favorites WHERE article_id = a.id AND user_id = "#,
        );

        query_builder.push_bind(viewer_id);
        query_builder.push(
            r#") AS favorited,
            (SELECT COUNT(*) FROM favorites WHERE article_id = a.id) AS favorites_count,
            COALESCE(ARRAY_AGG(t.name) FILTER (WHERE t.name IS NOT NULL), '{}') AS tag_list
        FROM articles a
        INNER JOIN users u ON a.author_id = u.id
        LEFT JOIN follows f ON f.followed_id = u.id AND f.follower_id = "#,
        );

        query_builder.push_bind(viewer_id);

        query_builder.push(" LEFT JOIN article_tags at ON at.article_id = a.id ");
        query_builder.push(" LEFT JOIN tags t ON t.id = at.tag_id ");

        query_builder.push(" WHERE 1=1 ");

        if let Some(tag) = filter.tag {
            query_builder.push(" AND a.id IN (SELECT article_id FROM article_tags at2 JOIN tags t2 ON t2.id = at2.tag_id WHERE t2.name = ");
            query_builder.push_bind(tag);
            query_builder.push(") ");
        }

        if let Some(author) = filter.author {
            query_builder.push(" AND u.username = ");
            query_builder.push_bind(author);
        }

        if let Some(favorited_by) = filter.favorited {
            query_builder.push(" AND a.id IN ( ");
            query_builder.push("   SELECT fav.article_id FROM favorites fav ");
            query_builder.push("   JOIN users u2 ON u2.id = fav.user_id ");
            query_builder.push("   WHERE u2.username = ");
            query_builder.push_bind(favorited_by);
            query_builder.push(" ) ");
        }

        query_builder.push(" GROUP BY a.id, u.id, f.follower_id ");
        query_builder.push(" ORDER BY a.created_at DESC ");

        if let Some(limit) = filter.limit {
            query_builder.push(" LIMIT ");
            query_builder.push_bind(limit as i64);
        }

        if let Some(offset) = filter.offset {
            query_builder.push(" OFFSET ");
            query_builder.push_bind(offset as i64);
        }

        let rows: Vec<ArticleRow> = query_builder
            .build_query_as::<ArticleRow>()
            .fetch_all(&self.pool)
            .await?;

        let total_count = rows.first().map(|r| r.total_count).unwrap_or(0);
        let articles = rows
            .into_iter()
            .map(|r| Article {
                id: r.id,
                slug: r.slug,
                title: r.title,
                description: r.description,
                body: r.body,
                tag_list: r.tag_list,
                created_at: r.created_at,
                updated_at: r.updated_at,
                favorited: r.favorited,
                favorites_count: r.favorites_count as u32,
                author_id: r.author_id,
                author: Profile {
                    username: r.author_username,
                    bio: r.author_bio,
                    image: r.author_image,
                    following: r.following,
                },
            })
            .collect();

        Ok((articles, total_count))
    }

    async fn update(
        &self,
        slug: &str,
        author_id: Uuid,
        title: Option<String>,
        description: Option<String>,
        body: Option<String>,
        tag_list: Option<Option<Vec<String>>>,
    ) -> Result<Article, AppError> {
        let mut tx = self.pool.begin().await?;

        let new_slug = title.as_ref().map(|t| Article::generate_slug(t));
        let row = sqlx::query!(
            r#"
            UPDATE articles 
            SET 
                title = COALESCE($3, title),
                slug = COALESCE($4, slug), -- Обновляем slug, если он пришел
                description = COALESCE($5, description),
                body = COALESCE($6, body),
                updated_at = NOW()
            WHERE slug = $1 AND author_id = $2
            RETURNING id, slug
            "#,
            slug,
            author_id,
            title,
            new_slug,
            description,
            body
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::ArticleNotFound)?;

        if let Some(maybe_tags) = tag_list {
            match maybe_tags {
                Some(tags) => {
                    sqlx::query!("DELETE FROM article_tags WHERE article_id = $1", row.id)
                        .execute(&mut *tx)
                        .await?;

                    for tag_name in tags {
                        let tag_id = sqlx::query!(
                        "INSERT INTO tags (name) VALUES ($1) ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name RETURNING id",
                        tag_name
                    ).fetch_one(&mut *tx).await?.id;

                        sqlx::query!(
                            "INSERT INTO article_tags (article_id, tag_id) VALUES ($1, $2)",
                            row.id,
                            tag_id
                        )
                        .execute(&mut *tx)
                        .await?;
                    }
                }
                None => {
                    return Err(AppError::ValidationError(HashMap::from([(
                        "tagList".to_string(),
                        vec!["can't be null".to_string()],
                    )])));
                }
            }
        }

        tx.commit().await?;

        self.get_by_slug(&row.slug, Some(author_id))
            .await?
            .ok_or(AppError::ArticleNotFound)
    }

    async fn get_feed(
        &self,
        viewer_id: Uuid,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<Article>, i64), AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                a.*, 
                u.username AS author_username, u.bio AS author_bio, u.image AS author_image,
                true AS "following!",
                (SELECT COUNT(*) FROM favorites WHERE article_id = a.id) AS "favorites_count!",
                EXISTS(SELECT 1 FROM favorites WHERE article_id = a.id AND user_id = $1) AS "favorited!",
                COALESCE(ARRAY_AGG(t.name) FILTER (WHERE t.name IS NOT NULL), '{}') AS "tag_list!",
                COUNT(*) OVER() AS "total_count!" 
            FROM articles a
            INNER JOIN users u ON a.author_id = u.id
            INNER JOIN follows f ON f.followed_id = u.id AND f.follower_id = $1
            LEFT JOIN article_tags at ON at.article_id = a.id
            LEFT JOIN tags t ON t.id = at.tag_id
            GROUP BY a.id, u.id
            ORDER BY a.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            viewer_id,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?;

        let total_count = rows.first().map(|r| r.total_count).unwrap_or(0);
        let articles = rows
            .into_iter()
            .map(|r| Article {
                id: r.id,
                slug: r.slug,
                title: r.title,
                description: r.description,
                body: r.body,
                tag_list: r.tag_list,
                created_at: r.created_at,
                updated_at: r.updated_at,
                favorited: r.favorited,
                favorites_count: r.favorites_count as u32,
                author_id: r.author_id,
                author: Profile {
                    username: r.author_username,
                    bio: r.author_bio,
                    image: r.author_image,
                    following: r.following,
                },
            })
            .collect();
        Ok((articles, total_count))
    }

    async fn favorite(&self, slug: &str, user_id: Uuid) -> Result<Article, AppError> {
        let article_id = sqlx::query!("SELECT id FROM articles WHERE slug = $1", slug)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AppError::ArticleNotFound)?
            .id;

        sqlx::query!(
            "INSERT INTO favorites (user_id, article_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            user_id,
            article_id
        )
        .execute(&self.pool)
        .await?;

        self.get_by_slug(slug, Some(user_id))
            .await?
            .ok_or(AppError::ArticleNotFound)
    }

    async fn unfavorite(&self, slug: &str, user_id: Uuid) -> Result<Article, AppError> {
        let article_id = sqlx::query!("SELECT id FROM articles WHERE slug = $1", slug)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AppError::ArticleNotFound)?
            .id;

        sqlx::query!(
            "DELETE FROM favorites WHERE user_id = $1 AND article_id = $2",
            user_id,
            article_id
        )
        .execute(&self.pool)
        .await?;

        self.get_by_slug(slug, Some(user_id))
            .await?
            .ok_or(AppError::ArticleNotFound)
    }

    async fn get_all_tags(&self) -> Result<Vec<String>, AppError> {
        let tags = sqlx::query!("SELECT name FROM tags ORDER BY name ASC")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| r.name)
            .collect();

        Ok(tags)
    }
}
