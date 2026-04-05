use crate::{application::services::user_service::Claims, domain::errors::AppError};
use axum::{extract::FromRequestParts, http::request::Parts};
use jsonwebtoken::{DecodingKey, Validation, decode};
use uuid::Uuid;

pub trait AuthConfig {
    fn jwt_secret(&self) -> &str;
}

pub struct AuthUser {
    pub user_id: Uuid,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync + AuthConfig,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        tracing::info!("{:?}", parts);
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::MissingToken)?;

        if !auth_header.starts_with("Token ") {
            return Err(AppError::MissingToken);
        }

        let token = &auth_header[6..];

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.jwt_secret().as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::MissingToken)?;

        Ok(AuthUser {
            user_id: token_data.claims.sub,
        })
    }
}

pub struct OptionalAuthUser(pub Option<uuid::Uuid>);

impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync + AuthConfig,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers.get(axum::http::header::AUTHORIZATION);

        if let Some(header) = auth_header {
            let header_str = header.to_str().map_err(|_| AppError::MissingToken)?;
            if !header_str.starts_with("Token ") {
                return Err(AppError::MissingToken);
            }

            let token = &header_str[6..];

            let token_data = decode::<Claims>(
                token,
                &DecodingKey::from_secret(state.jwt_secret().as_bytes()),
                &Validation::default(),
            )
            .map_err(|_| AppError::MissingToken)?;

            return Ok(OptionalAuthUser(Some(token_data.claims.sub)));
        }

        Ok(OptionalAuthUser(None))
    }
}
