use crate::domain::errors::AppError;
use crate::domain::models::article::{Article, ArticleFilter};
use crate::domain::models::comment::Comment;
use crate::domain::models::profile::Profile;
use crate::domain::models::user::{User, UserPasswordHash};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User, password_hash: &UserPasswordHash) -> Result<(), AppError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<(User, UserPasswordHash)>, AppError>;

    async fn find_by_email(
        &self,
        email: &str,
    ) -> Result<Option<(User, UserPasswordHash)>, AppError>;

    async fn find_by_username(
        &self,
        email: &str,
    ) -> Result<Option<(User, UserPasswordHash)>, AppError>;

    async fn update(&self, user: &User) -> Result<User, AppError>;

    async fn update_password_hash(
        &self,
        user_id: Uuid,
        password_hash: &UserPasswordHash,
    ) -> Result<(), AppError>;
}

#[async_trait]
pub trait ProfileRepository: Send + Sync {
    async fn get_profile(
        &self,
        target_username: &str,
        viewer_id: Option<Uuid>,
    ) -> Result<Option<Profile>, AppError>;

    async fn add_follow(&self, follower_id: Uuid, followed_id: Uuid) -> Result<(), AppError>;

    async fn remove_follow(&self, follower_id: Uuid, followed_id: Uuid) -> Result<(), AppError>;
}

#[async_trait]
pub trait ArticleRepository: Send + Sync {
    async fn create(
        &self,
        slug: &str,
        title: &str,
        description: &str,
        body: &str,
        author_id: Uuid,
        tags: &[String],
    ) -> Result<Article, AppError>;
    async fn get_by_slug(
        &self,
        slug: &str,
        viewer_id: Option<Uuid>,
    ) -> Result<Option<Article>, AppError>;
    async fn list_articles(
        &self,
        filter: ArticleFilter,
        viewer_id: Option<Uuid>,
    ) -> Result<(Vec<Article>, i64), AppError>;
    async fn update(
        &self,
        slug: &str,
        author_id: Uuid,
        title: Option<String>,
        description: Option<String>,
        body: Option<String>,
        tag_list: Option<Option<Vec<String>>>,
    ) -> Result<Article, AppError>;
    async fn delete(&self, slug: &str, author_id: Uuid) -> Result<(), AppError>;
    async fn get_feed(
        &self,
        viewer_id: Uuid,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<Article>, i64), AppError>;
    async fn favorite(&self, slug: &str, user_id: Uuid) -> Result<Article, AppError>;
    async fn unfavorite(&self, slug: &str, user_id: Uuid) -> Result<Article, AppError>;
    async fn get_all_tags(&self) -> Result<Vec<String>, AppError>;
}

#[async_trait]
pub trait CommentRepository: Send + Sync {
    async fn add_comment(
        &self,
        article_slug: &str,
        author_id: Uuid,
        body: &str,
    ) -> Result<Comment, AppError>;
    async fn get_comments_by_article(
        &self,
        article_slug: &str,
        viewer_id: Option<Uuid>,
    ) -> Result<Vec<Comment>, AppError>;
    async fn delete_comment(&self, comment_id: i32, author_id: Uuid) -> Result<(), AppError>;
    async fn get_comment_author_id(&self, comment_id: i32) -> Result<Option<Uuid>, AppError>;
}
