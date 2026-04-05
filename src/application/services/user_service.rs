use std::sync::Arc;

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use serde_email::Email;
use tracing::error;
use uuid::Uuid;

use crate::{
    api::dto::UpdateUserInner,
    domain::{
        errors::AppError,
        models::user::{User, UserPasswordHash},
        repositories::UserRepository,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: uuid::Uuid,
    pub exp: i64,
}

pub struct UserService {
    pub user_repo: Arc<dyn UserRepository>,
    pub jwt_secret: String,
    jwt_exp_hours: u64,
}

impl UserService {
    pub fn new(user_repo: Arc<dyn UserRepository>, jwt_secret: String, jwt_exp_hours: u64) -> Self {
        Self {
            user_repo,
            jwt_secret,
            jwt_exp_hours,
        }
    }

    pub async fn register(
        &self,
        username: String,
        email: Email,
        password_raw: String,
    ) -> Result<(User, String), AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash_string = argon2
            .hash_password(password_raw.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(e.to_string()))?
            .to_string();
        let hashed_password = UserPasswordHash(password_hash_string);
        let user = User::new(username, email);
        self.user_repo.create(&user, &hashed_password).await?;

        let token = self.generate_token(user.id)?;

        Ok((user, token))
    }

    pub async fn login(
        &self,
        email: String,
        password_raw: String,
    ) -> Result<(User, String), AppError> {
        let (user, stored_hash) = self
            .user_repo
            .find_by_email(&email)
            .await?
            .ok_or(AppError::UserNotFound)?;

        let parsed_hash = PasswordHash::new(&stored_hash.0).map_err(|e| {
            error!("Invalid hash format: {:?}", e);
            AppError::Internal("Invalid hash format".into())
        })?;

        Argon2::default()
            .verify_password(password_raw.as_bytes(), &parsed_hash)
            .map_err(|e| {
                error!("Invalid password: {:?}", e);
                AppError::AuthError
            })?;

        let token = self.generate_token(user.id)?;

        Ok((user, token))
    }

    fn generate_token(&self, user_id: uuid::Uuid) -> Result<String, AppError> {
        let expiration = Utc::now() + Duration::hours(self.jwt_exp_hours as i64);
        let claims = Claims {
            sub: user_id,
            exp: expiration.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(e.to_string()))
    }

    pub fn jwt_secret(&self) -> &str {
        &self.jwt_secret
    }

    pub async fn get_current_user(&self, user_id: Uuid) -> Result<(User, String), AppError> {
        let (user, _) = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(AppError::UserNotFound)?;

        let token = self.generate_token(user_id)?;
        Ok((user, token))
    }

    pub async fn update_user(
        &self,
        user_id: Uuid,
        payload: UpdateUserInner,
    ) -> Result<(User, String), AppError> {
        let (mut user, _) = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(AppError::UserNotFound)?;

        if let Some(email) = payload.email {
            if let Some(email) = email {
                user.email = email.parse().unwrap();
            } else {
                return Err(AppError::bad_request("email", "can't be blank"));
            };
        };

        if let Some(username) = payload.username {
            if let Some(username) = username {
                user.username = username
            } else {
                return Err(AppError::bad_request("username", "can't be blank"));
            };
        };

        if let Some(bio_option) = payload.bio {
            user.bio = match bio_option {
                Some(s) if s.is_empty() => None,
                Some(s) => Some(s),
                None => None,
            }
        }

        if let Some(image_option) = payload.image {
            user.image = match image_option {
                Some(s) if s.is_empty() => None,
                Some(s) => Some(s),
                None => None,
            }
        }

        if let Some(password_raw) = payload.password {
            if let Some(password) = password_raw {
                let salt = SaltString::generate(&mut OsRng);
                let hashed = Argon2::default()
                    .hash_password(password.as_bytes(), &salt)
                    .map_err(|e| AppError::Internal(e.to_string()))?
                    .to_string();

                self.user_repo
                    .update_password_hash(user.id, &UserPasswordHash(hashed))
                    .await?;
            } else {
                return Err(AppError::bad_request("password", "can't be blank"));
            };
        }

        let updated_user = self.user_repo.update(&user).await?;
        let token = self.generate_token(updated_user.id)?;

        Ok((updated_user, token))
    }
}
