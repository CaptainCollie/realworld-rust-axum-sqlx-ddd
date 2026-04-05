use crate::api::dto::{CreateArticleInner, UpdateArticleInner};
use crate::domain::{
    errors::AppError,
    models::article::{Article, ArticleFilter},
    repositories::ArticleRepository,
};
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

pub struct ArticleService {
    article_repo: Arc<dyn ArticleRepository>,
}

impl ArticleService {
    pub fn new(article_repo: Arc<dyn ArticleRepository>) -> Self {
        Self { article_repo }
    }

    #[instrument(skip(self, article), fields(title = %article.title, author_id = %author_id))]
    pub async fn create_article(
        &self,
        author_id: Uuid,
        article: CreateArticleInner,
    ) -> Result<Article, AppError> {
        let slug = Article::generate_slug(&article.title);
        let tags = article.tag_list.unwrap_or_default();

        self.article_repo
            .create(
                &slug,
                &article.title,
                &article.description,
                &article.body,
                author_id,
                &tags,
            )
            .await
    }

    #[instrument(skip(self), fields(slug = %slug, viewer = ?viewer_id))]
    pub async fn get_article(
        &self,
        slug: &str,
        viewer_id: Option<Uuid>,
    ) -> Result<Article, AppError> {
        self.article_repo
            .get_by_slug(slug, viewer_id)
            .await?
            .ok_or(AppError::ArticleNotFound)
    }

    #[instrument(skip(self), fields(slug = %slug, author = %author_id))]
    pub async fn delete_article(&self, slug: &str, author_id: Uuid) -> Result<(), AppError> {
        let article = self
            .article_repo
            .get_by_slug(slug, None)
            .await?
            .ok_or(AppError::ArticleNotFound)?;

        if article.author_id != author_id {
            return Err(AppError::ArticleForbidden);
        }

        self.article_repo.delete(slug, author_id).await
    }

    #[instrument(skip(self, filter), fields(filter = ?filter, viewer = ?viewer_id))]
    pub async fn list_articles(
        &self,
        filter: ArticleFilter,
        viewer_id: Option<Uuid>,
    ) -> Result<(Vec<Article>, i64), AppError> {
        self.article_repo.list_articles(filter, viewer_id).await
    }

    #[instrument(skip(self, article_update), fields(slug = %slug, author_id = %author_id))]
    pub async fn update_article(
        &self,
        slug: &str,
        author_id: Uuid,
        article_update: UpdateArticleInner,
    ) -> Result<Article, AppError> {
        let article = self
            .article_repo
            .get_by_slug(slug, None)
            .await?
            .ok_or(AppError::ArticleNotFound)?;

        if article.author_id != author_id {
            return Err(AppError::ArticleForbidden);
        }

        self.article_repo
            .update(
                slug,
                author_id,
                article_update.title,
                article_update.description,
                article_update.body,
                article_update.tag_list,
            )
            .await
    }

    #[instrument(skip(self), fields(viewer_id = %viewer_id))]
    pub async fn get_feed(
        &self,
        viewer_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<(Vec<Article>, i64), AppError> {
        self.article_repo
            .get_feed(viewer_id, limit.unwrap_or(20), offset.unwrap_or(0))
            .await
    }

    #[instrument(skip(self), fields(slug = %slug, user_id = %user_id))]
    pub async fn favorite_article(&self, slug: &str, user_id: Uuid) -> Result<Article, AppError> {
        self.article_repo.favorite(slug, user_id).await
    }

    #[instrument(skip(self), fields(slug = %slug, user_id = %user_id))]
    pub async fn unfavorite_article(&self, slug: &str, user_id: Uuid) -> Result<Article, AppError> {
        self.article_repo.unfavorite(slug, user_id).await
    }

    #[instrument(skip(self))]
    pub async fn get_tags(&self) -> Result<Vec<String>, AppError> {
        self.article_repo.get_all_tags().await
    }
}
