use std::sync::Arc;

use tracing::instrument;
use uuid::Uuid;

use crate::domain::{
    errors::AppError,
    models::profile::Profile,
    repositories::{ProfileRepository, UserRepository},
};

pub struct ProfileService {
    profile_repo: Arc<dyn ProfileRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl ProfileService {
    pub fn new(
        profile_repo: Arc<dyn ProfileRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            profile_repo,
            user_repo,
        }
    }

    #[instrument(skip(self), fields(username = %username, viewer = ?viewer_id))]
    pub async fn get_profile(
        &self,
        username: &str,
        viewer_id: Option<Uuid>,
    ) -> Result<Profile, AppError> {
        self.profile_repo
            .get_profile(username, viewer_id)
            .await?
            .ok_or(AppError::ProfileNotFound)
    }

    #[instrument(skip(self), fields(target = %target_username, follower = %follower_id))]
    pub async fn follow(
        &self,
        target_username: &str,
        follower_id: Uuid,
    ) -> Result<Profile, AppError> {
        let (target_user, _) = self
            .user_repo
            .find_by_username(target_username)
            .await?
            .ok_or(AppError::ProfileNotFound)?;

        if target_user.id == follower_id {
            return Err(AppError::Conflict {
                field: "body".to_string(),
                message: "You cannot follow yourself".to_string(),
            });
        }

        self.profile_repo
            .add_follow(follower_id, target_user.id)
            .await?;

        self.get_profile(&target_user.username, Some(follower_id))
            .await
    }

    #[instrument(skip(self), fields(target = %target_username, follower = %follower_id))]
    pub async fn unfollow(
        &self,
        target_username: &str,
        follower_id: Uuid,
    ) -> Result<Profile, AppError> {
        let (target_user, _) = self
            .user_repo
            .find_by_username(target_username)
            .await?
            .ok_or(AppError::ProfileNotFound)?;

        self.profile_repo
            .remove_follow(follower_id, target_user.id)
            .await?;

        self.get_profile(target_username, Some(follower_id)).await
    }
}
