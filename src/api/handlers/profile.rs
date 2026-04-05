use axum::extract::Path;
use axum::{Json, extract::State, response::IntoResponse};

use crate::api::dto::{ProfileResponse, ProfileResponseInner};
use crate::api::extractor::AuthUser;
use crate::api::{AppState, extractor::OptionalAuthUser};
use crate::domain::errors::AppError;

pub async fn get_profile(
    State(state): State<AppState>,
    OptionalAuthUser(viewer_id): OptionalAuthUser,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let profile = state
        .profile_service
        .get_profile(&username, viewer_id)
        .await?;
    Ok(Json(ProfileResponse {
        profile: ProfileResponseInner {
            username: profile.username,
            bio: profile.bio,
            image: profile.image,
            following: profile.following,
        },
    }))
}

pub async fn follow_user(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let profile = state
        .profile_service
        .follow(&username, auth_user.user_id)
        .await?;
    Ok(Json(ProfileResponse {
        profile: ProfileResponseInner {
            username: profile.username,
            bio: profile.bio,
            image: profile.image,
            following: profile.following,
        },
    }))
}

pub async fn unfollow_user(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let profile = state
        .profile_service
        .unfollow(&username, auth_user.user_id)
        .await?;
    Ok(Json(ProfileResponse {
        profile: ProfileResponseInner {
            username: profile.username,
            bio: profile.bio,
            image: profile.image,
            following: profile.following,
        },
    }))
}
