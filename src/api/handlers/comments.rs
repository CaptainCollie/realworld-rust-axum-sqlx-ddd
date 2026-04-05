use crate::{
    api::{
        AppState,
        dto::{
            CommentResponse, CommentResponseInner, CommentsResponse, CreateCommentRequest,
            ProfileResponseInner,
        },
        extractor::{AuthUser, OptionalAuthUser},
    },
    domain::errors::AppError,
};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use reqwest::StatusCode;
use validator::Validate;

pub async fn add_comment(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    auth_user: AuthUser,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(AppError::from_validation)?;

    let comment = state
        .comment_service
        .add_comment(&slug, auth_user.user_id, payload.comment)
        .await?;

    let response = CommentResponse {
        comment: CommentResponseInner {
            id: comment.id,
            created_at: comment.created_at,
            updated_at: comment.updated_at,
            body: comment.body,
            author: ProfileResponseInner {
                username: comment.author.username,
                bio: comment.author.bio,
                image: comment.author.image,
                following: comment.author.following,
            },
        },
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_comments(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    opt_auth: OptionalAuthUser,
) -> Result<impl IntoResponse, AppError> {
    let comments = state
        .comment_service
        .get_comments(&slug, opt_auth.0)
        .await?;
    let comments = comments
        .into_iter()
        .map(|comment| CommentResponseInner {
            id: comment.id,
            created_at: comment.created_at,
            updated_at: comment.updated_at,
            body: comment.body,
            author: ProfileResponseInner {
                username: comment.author.username,
                bio: comment.author.bio,
                image: comment.author.image,
                following: comment.author.following,
            },
        })
        .collect();

    Ok(Json(CommentsResponse { comments }))
}

pub async fn delete_comment(
    State(state): State<AppState>,
    Path((slug, comment_id)): Path<(String, i32)>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    state
        .comment_service
        .delete_comment(&slug, comment_id, auth_user.user_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
