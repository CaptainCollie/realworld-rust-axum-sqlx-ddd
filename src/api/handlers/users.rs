use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::api::{
    AppState,
    dto::{
        LoginUserRequest, RegisterUserRequest, UpdateUserRequest, UserResponse, UserResponseInner,
    },
    extractor::AuthUser,
};
use crate::domain::errors::AppError;
use validator::Validate;

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(AppError::from_validation)?;

    let email = payload
        .user
        .email
        .parse()
        .map_err(|_| AppError::Internal("Invalid email format".into()))?;

    let (user, token) = state
        .user_service
        .register(payload.user.username, email, payload.user.password)
        .await
        .map_err(|e| {
            tracing::error!("DETAILED ERROR: {:?}", e);
            e
        })?;

    let response = UserResponse {
        user: UserResponseInner {
            email: user.email.to_string(),
            token,
            username: user.username,
            bio: user.bio,
            image: user.image,
        },
    };

    Ok((StatusCode::CREATED, axum::Json(response)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(AppError::from_validation)?;

    let email = payload
        .user
        .email
        .parse()
        .map_err(|_| AppError::Internal("Invalid email format".into()))?;

    let (user, token) = state
        .user_service
        .login(email, payload.user.password)
        .await?;

    let response = UserResponse {
        user: UserResponseInner {
            email: user.email.to_string(),
            token,
            username: user.username,
            bio: user.bio,
            image: user.image,
        },
    };

    Ok((StatusCode::OK, axum::Json(response)))
}

pub async fn get_current_user(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let (user, token) = state
        .user_service
        .get_current_user(auth_user.user_id)
        .await?;

    let response = UserResponse {
        user: UserResponseInner {
            email: user.email.to_string(),
            token,
            username: user.username,
            bio: user.bio,
            image: user.image,
        },
    };

    Ok((StatusCode::OK, axum::Json(response)))
}

pub async fn update_current_user(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(AppError::from_validation)?;

    let (user, token) = state
        .user_service
        .update_user(auth_user.user_id, payload.user)
        .await
        .map_err(|e| {
            tracing::error!("UPDATE_USER_ERROR: {:?}", e);
            e
        })?;

    let response = UserResponse {
        user: UserResponseInner {
            email: user.email.to_string(),
            token,
            username: user.username,
            bio: user.bio,
            image: user.image,
        },
    };

    Ok(Json(response))
}
