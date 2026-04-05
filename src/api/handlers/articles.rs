use crate::api::AppState;
use crate::api::dto::{
    ArticleListResponse, ArticleResponse, ArticleResponseInner, CreateArticleRequest,
    ProfileResponseInner, TagsResponse, UpdateArticleRequest,
};
use crate::api::extractor::{AuthUser, OptionalAuthUser};
use crate::domain::errors::AppError;
use crate::domain::models::article::{ArticleFilter, PaginationParams};
use axum::extract::{Path, Query};
use axum::{Json, extract::State, response::IntoResponse};
use reqwest::StatusCode;
use validator::Validate;

pub async fn create_article(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateArticleRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(AppError::from_validation)?;

    let article = state
        .article_service
        .create_article(auth_user.user_id, payload.article)
        .await?;

    let response = ArticleResponse {
        article: ArticleResponseInner {
            slug: article.slug,
            title: article.title,
            description: article.description,
            body: Some(article.body),
            tag_list: article.tag_list,
            created_at: article.created_at,
            updated_at: article.updated_at,
            favorited: article.favorited,
            favorites_count: article.favorites_count,
            author: ProfileResponseInner {
                username: article.author.username,
                bio: article.author.bio,
                image: article.author.image,
                following: article.author.following,
            },
        },
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_article(
    State(state): State<AppState>,
    OptionalAuthUser(viewer_id): OptionalAuthUser,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let article = state.article_service.get_article(&slug, viewer_id).await?;

    let response = ArticleResponse {
        article: ArticleResponseInner {
            slug: article.slug,
            title: article.title,
            description: article.description,
            body: Some(article.body),
            tag_list: article.tag_list,
            created_at: article.created_at,
            updated_at: article.updated_at,
            favorited: article.favorited,
            favorites_count: article.favorites_count,
            author: ProfileResponseInner {
                username: article.author.username,
                bio: article.author.bio,
                image: article.author.image,
                following: article.author.following,
            },
        },
    };

    Ok(Json(response))
}

pub async fn delete_article(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    state
        .article_service
        .delete_article(&slug, auth_user.user_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_articles(
    State(state): State<AppState>,
    OptionalAuthUser(viewer_id): OptionalAuthUser,
    Query(filter): Query<ArticleFilter>,
) -> Result<impl IntoResponse, AppError> {
    let (articles, articles_count) = state
        .article_service
        .list_articles(filter, viewer_id)
        .await?;

    let response = ArticleListResponse {
        articles: articles
            .into_iter()
            .map(|article| ArticleResponseInner {
                slug: article.slug,
                title: article.title,
                description: article.description,
                body: None,
                tag_list: article.tag_list,
                created_at: article.created_at,
                updated_at: article.updated_at,
                favorited: article.favorited,
                favorites_count: article.favorites_count,
                author: ProfileResponseInner {
                    username: article.author.username,
                    bio: article.author.bio,
                    image: article.author.image,
                    following: article.author.following,
                },
            })
            .collect(),
        articles_count: articles_count as usize,
    };

    Ok(Json(response))
}

pub async fn update_article(
    auth_user: AuthUser,
    Path(slug): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateArticleRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(AppError::from_validation)?;
    tracing::info!("{payload:?}");
    let article = state
        .article_service
        .update_article(&slug, auth_user.user_id, payload.article)
        .await?;

    let response = ArticleResponse {
        article: ArticleResponseInner {
            slug: article.slug,
            title: article.title,
            description: article.description,
            body: Some(article.body),
            tag_list: article.tag_list,
            created_at: article.created_at,
            updated_at: article.updated_at,
            favorited: article.favorited,
            favorites_count: article.favorites_count,
            author: ProfileResponseInner {
                username: article.author.username,
                bio: article.author.bio,
                image: article.author.image,
                following: article.author.following,
            },
        },
    };

    Ok(Json(response))
}

pub async fn get_feed(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let (articles, articles_count) = state
        .article_service
        .get_feed(auth_user.user_id, pagination.limit, pagination.offset)
        .await?;

    Ok(Json(ArticleListResponse {
        articles: articles
            .into_iter()
            .map(|article| ArticleResponseInner {
                slug: article.slug,
                title: article.title,
                description: article.description,
                body: None,
                tag_list: article.tag_list,
                created_at: article.created_at,
                updated_at: article.updated_at,
                favorited: article.favorited,
                favorites_count: article.favorites_count,
                author: ProfileResponseInner {
                    username: article.author.username,
                    bio: article.author.bio,
                    image: article.author.image,
                    following: article.author.following,
                },
            })
            .collect(),
        articles_count: articles_count as usize,
    }))
}

pub async fn favorite_article(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let article = state
        .article_service
        .favorite_article(&slug, auth_user.user_id)
        .await?;

    let response = ArticleResponse {
        article: ArticleResponseInner {
            slug: article.slug,
            title: article.title,
            description: article.description,
            body: Some(article.body),
            tag_list: article.tag_list,
            created_at: article.created_at,
            updated_at: article.updated_at,
            favorited: article.favorited,
            favorites_count: article.favorites_count,
            author: ProfileResponseInner {
                username: article.author.username,
                bio: article.author.bio,
                image: article.author.image,
                following: article.author.following,
            },
        },
    };

    Ok(Json(response))
}

pub async fn unfavorite_article(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let article = state
        .article_service
        .unfavorite_article(&slug, auth_user.user_id)
        .await?;

    let response = ArticleResponse {
        article: ArticleResponseInner {
            slug: article.slug,
            title: article.title,
            description: article.description,
            body: Some(article.body),
            tag_list: article.tag_list,
            created_at: article.created_at,
            updated_at: article.updated_at,
            favorited: article.favorited,
            favorites_count: article.favorites_count,
            author: ProfileResponseInner {
                username: article.author.username,
                bio: article.author.bio,
                image: article.author.image,
                following: article.author.following,
            },
        },
    };

    Ok(Json(response))
}

pub async fn get_tags(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let tags = state.article_service.get_tags().await?;

    let response = TagsResponse { tags };

    Ok(Json(response))
}
