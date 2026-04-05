use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterUserRequest {
    #[validate(nested)]
    pub user: RegisterUserInner,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterUserInner {
    #[validate(length(min = 1, message = "can't be blank"))]
    pub username: String,

    #[validate(
        length(min = 1, message = "can't be blank"),
        email(message = "is invalid")
    )]
    pub email: String,

    #[validate(length(min = 1, message = "can't be blank"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginUserRequest {
    #[validate(nested)]
    pub user: LoginUserInner,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginUserInner {
    #[validate(
        length(min = 1, message = "can't be blank"),
        email(message = "is invalid")
    )]
    pub email: String,

    #[validate(length(min = 1, message = "can't be blank"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub user: UserResponseInner,
}

#[derive(Debug, Serialize)]
pub struct UserResponseInner {
    pub email: String,
    pub token: String,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(nested)]
    pub user: UpdateUserInner,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserInner {
    #[serde(default, deserialize_with = "deserialize_resettable")]
    #[validate(
        length(min = 1, message = "can't be blank"),
        email(message = "is invalid")
    )]
    pub email: Option<Option<String>>,

    #[serde(default, deserialize_with = "deserialize_resettable")]
    #[validate(length(min = 1, message = "can't be blank"))]
    pub username: Option<Option<String>>,

    #[serde(default, deserialize_with = "deserialize_resettable")]
    #[validate(length(min = 1, message = "can't be blank"))]
    pub password: Option<Option<String>>,

    #[serde(default, deserialize_with = "deserialize_resettable")]
    pub bio: Option<Option<String>>,

    #[serde(default, deserialize_with = "deserialize_resettable")]
    pub image: Option<Option<String>>,
}

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub profile: ProfileResponseInner,
}

#[derive(Debug, Serialize)]
pub struct ProfileResponseInner {
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub following: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateArticleRequest {
    #[validate(nested)]
    pub article: CreateArticleInner,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateArticleInner {
    #[validate(length(min = 1, message = "can't be blank"))]
    pub title: String,

    #[validate(length(min = 1, message = "can't be blank"))]
    pub description: String,

    #[validate(length(min = 1, message = "can't be blank"))]
    pub body: String,

    #[serde(rename = "tagList")]
    pub tag_list: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct UpdateArticleRequest {
    #[validate(nested)]
    pub article: UpdateArticleInner,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct UpdateArticleInner {
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,

    #[serde(rename = "tagList")]
    #[serde(default, deserialize_with = "deserialize_resettable")]
    pub tag_list: Option<Option<Vec<String>>>,
}

#[derive(Debug, Serialize)]
pub struct ArticleResponse {
    pub article: ArticleResponseInner,
}

#[derive(Debug, Serialize)]
pub struct ArticleListResponse {
    pub articles: Vec<ArticleResponseInner>,

    #[serde(rename = "articlesCount")]
    pub articles_count: usize,
}

#[derive(Debug, Serialize)]
pub struct TagsResponse {
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ArticleResponseInner {
    pub slug: String,
    pub title: String,
    pub description: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    #[serde(rename = "tagList")]
    pub tag_list: Vec<String>,

    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,

    pub favorited: bool,

    #[serde(rename = "favoritesCount")]
    pub favorites_count: u32,

    pub author: ProfileResponseInner,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCommentRequest {
    #[validate(nested)]
    pub comment: CreateCommentInner,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCommentInner {
    #[validate(length(min = 1, message = "can't be blank"))]
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct CommentResponse {
    pub comment: CommentResponseInner,
}

#[derive(Debug, Serialize)]
pub struct CommentsResponse {
    pub comments: Vec<CommentResponseInner>,
}

#[derive(Debug, Serialize)]
pub struct CommentResponseInner {
    pub id: i32,

    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,

    pub body: String,

    pub author: ProfileResponseInner,
}

fn deserialize_resettable<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::<T>::deserialize(deserializer)?))
}
