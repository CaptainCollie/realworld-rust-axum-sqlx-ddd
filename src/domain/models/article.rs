use crate::domain::models::profile::Profile;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use slug::slugify;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub tag_list: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub favorited: bool,
    pub favorites_count: u32,
    pub author_id: Uuid,
    pub author: Profile,
}

impl Article {
    pub fn generate_slug(title: &str) -> String {
        let slug = slugify(title);

        let suffix = uuid::Uuid::new_v4().to_string()[..8].to_string();

        if slug.is_empty() {
            suffix
        } else {
            format!("{}-{}", slug, suffix)
        }
    }
}
#[derive(Debug, Default, Deserialize)]
pub struct ArticleFilter {
    pub tag: Option<String>,
    pub author: Option<String>,
    pub favorited: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
pub struct PaginationParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}
