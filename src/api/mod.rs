pub mod dto;
pub mod extractor;
pub mod handlers;

use crate::{
    api::{
        extractor::AuthConfig,
        handlers::{
            articles::{
                create_article, delete_article, favorite_article, get_article, get_feed, get_tags,
                list_articles, unfavorite_article, update_article,
            },
            comments::{add_comment, delete_comment, get_comments},
            profile::{follow_user, get_profile, unfollow_user},
            users::{get_current_user, login, register, update_current_user},
        },
    },
    application::services::{
        article_service::ArticleService, comment_service::CommentService,
        profile_service::ProfileService, user_service::UserService,
    },
};
use axum::{
    Router,
    routing::{delete, get, post},
};
use std::sync::Arc;

pub fn create_router(
    user_service: Arc<UserService>,
    profile_service: Arc<ProfileService>,
    article_service: Arc<ArticleService>,
    comment_service: Arc<CommentService>,
) -> Router {
    Router::new()
        .route("/api/users", post(register))
        .route("/api/users/login", post(login))
        .route("/api/user", get(get_current_user).put(update_current_user))
        .route("/api/profiles/{username}", get(get_profile))
        .route("/api/articles", get(list_articles).post(create_article))
        .route("/api/articles/feed", get(get_feed))
        .route(
            "/api/articles/{slug}",
            get(get_article).delete(delete_article).put(update_article),
        )
        .route(
            "/api/articles/{slug}/favorite",
            post(favorite_article).delete(unfavorite_article),
        )
        .route(
            "/api/profiles/{username}/follow",
            post(follow_user).delete(unfollow_user),
        )
        .route("/api/tags", get(get_tags))
        .route(
            "/api/articles/{slug}/comments",
            get(get_comments).post(add_comment),
        )
        .route("/api/articles/{slug}/comments/{id}", delete(delete_comment))
        .with_state(AppState {
            user_service,
            profile_service,
            article_service,
            comment_service,
        })
}

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub profile_service: Arc<ProfileService>,
    pub article_service: Arc<ArticleService>,
    pub comment_service: Arc<CommentService>,
}

impl AuthConfig for AppState {
    fn jwt_secret(&self) -> &str {
        self.user_service.jwt_secret()
    }
}
