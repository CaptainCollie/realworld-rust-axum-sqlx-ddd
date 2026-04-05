use crate::api::dto::CreateCommentInner;
use crate::domain::repositories::ArticleRepository;
use crate::domain::{errors::AppError, models::comment::Comment, repositories::CommentRepository};
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

pub struct CommentService {
    comment_repo: Arc<dyn CommentRepository>,
    article_repo: Arc<dyn ArticleRepository>,
}

impl CommentService {
    pub fn new(
        comment_repo: Arc<dyn CommentRepository>,
        article_repo: Arc<dyn ArticleRepository>,
    ) -> Self {
        Self {
            comment_repo,
            article_repo,
        }
    }

    #[instrument(skip(self, dto), fields(slug = %slug, author_id = %author_id))]
    pub async fn add_comment(
        &self,
        slug: &str,
        author_id: Uuid,
        dto: CreateCommentInner,
    ) -> Result<Comment, AppError> {
        let _article = self
            .article_repo
            .get_by_slug(slug, None)
            .await?
            .ok_or(AppError::ArticleNotFound)?;

        self.comment_repo
            .add_comment(slug, author_id, &dto.body)
            .await
    }

    #[instrument(skip(self), fields(slug = %slug, viewer = ?viewer_id))]
    pub async fn get_comments(
        &self,
        slug: &str,
        viewer_id: Option<Uuid>,
    ) -> Result<Vec<Comment>, AppError> {
        let _article = self
            .article_repo
            .get_by_slug(slug, None)
            .await?
            .ok_or(AppError::ArticleNotFound)?;

        self.comment_repo
            .get_comments_by_article(slug, viewer_id)
            .await
    }

    #[instrument(skip(self), fields(slug = %slug, comment_id = %comment_id, user_id = %current_user_id))]
    pub async fn delete_comment(
        &self,
        slug: &str,
        comment_id: i32,
        current_user_id: Uuid,
    ) -> Result<(), AppError> {
        let _article = self
            .article_repo
            .get_by_slug(slug, None)
            .await?
            .ok_or(AppError::ArticleNotFound)?;

        let author_id = self
            .comment_repo
            .get_comment_author_id(comment_id)
            .await?
            .ok_or(AppError::CommentNotFound)?;

        if author_id != current_user_id {
            return Err(AppError::CommentForbidden);
        }

        self.comment_repo
            .delete_comment(comment_id, current_user_id)
            .await
    }
}
