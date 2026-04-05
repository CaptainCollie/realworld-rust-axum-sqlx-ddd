use serde::{Deserialize, Serialize};
use serde_email::Email;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: Email,
    pub bio: Option<String>,
    pub image: Option<String>,
}

impl User {
    pub fn new(username: String, email: Email) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            bio: None,
            image: None,
        }
    }
}

pub struct UserPasswordHash(pub String);
