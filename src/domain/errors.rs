use std::collections::HashMap;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Validation failed")]
    ValidationError(HashMap<String, Vec<String>>),

    #[error("Profile not found")]
    ProfileNotFound,

    #[error("Article not found")]
    ArticleNotFound,

    #[error("Comment not found")]
    CommentNotFound,

    #[error("Conflict: {field} {message}")]
    Conflict { field: String, message: String },

    #[error("Authentication failed")]
    AuthError,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Missing token")]
    MissingToken,

    #[error("Forbidden")]
    ArticleForbidden,

    #[error("Forbidden")]
    CommentForbidden,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::ValidationError(errs) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({ "errors": errs })),
            )
                .into_response(),
            AppError::Conflict { field, message } => (
                StatusCode::CONFLICT,
                Json(json!({ "errors": { field: [message] } })),
            )
                .into_response(),

            AppError::AuthError => (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "errors": { "credentials": ["invalid"] } })),
            )
                .into_response(),

            AppError::ProfileNotFound => (
                StatusCode::NOT_FOUND,
                Json(json!({ "errors": { "profile": ["not found"] } })),
            )
                .into_response(),
            AppError::ArticleNotFound => (
                StatusCode::NOT_FOUND,
                Json(json!({ "errors": { "article": ["not found"] } })),
            )
                .into_response(),
            AppError::CommentNotFound => (
                StatusCode::NOT_FOUND,
                Json(json!({ "errors": { "comment": ["not found"] } })),
            )
                .into_response(),
            AppError::MissingToken => (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "errors": { "token": ["is missing"] } })),
            )
                .into_response(),
            AppError::ArticleForbidden => (
                StatusCode::FORBIDDEN,
                Json(json!({ "errors": { "article": ["forbidden"] } })),
            )
                .into_response(),
            AppError::CommentForbidden => (
                StatusCode::FORBIDDEN,
                Json(json!({ "errors": { "comment": ["forbidden"] } })),
            )
                .into_response(),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "errors": { "body": ["internal server error"] } })),
            )
                .into_response(),
        }
    }
}

impl AppError {
    pub fn from_validation(e: validator::ValidationErrors) -> Self {
        let mut error_map = HashMap::new();
        Self::collect_validation_errors(&e, &mut error_map);
        AppError::ValidationError(error_map)
    }

    fn collect_validation_errors(
        current_errors: &validator::ValidationErrors,
        map: &mut HashMap<String, Vec<String>>,
    ) {
        for (field, error) in current_errors.errors() {
            match error {
                validator::ValidationErrorsKind::Field(field_errors) => {
                    if let Some(first_err) = field_errors.first() {
                        let message = first_err
                            .message
                            .as_ref()
                            .unwrap_or(&"is invalid".into())
                            .to_string();
                        map.insert(field.to_string(), vec![message]);
                    }
                }
                validator::ValidationErrorsKind::Struct(nested_errors) => {
                    Self::collect_validation_errors(nested_errors, map);
                }
                _ => (),
            }
        }
    }

    pub fn bad_request(field: &str, message: &str) -> Self {
        let mut map = std::collections::HashMap::new();
        map.insert(field.to_string(), vec![message.to_string()]);
        AppError::ValidationError(map)
    }
}
