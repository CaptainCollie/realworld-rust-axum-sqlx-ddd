#![allow(dead_code)]
use axum::body::Body;
use axum::http::Request;
use axum_test::TestServer;
use chrono::DateTime;
use chrono::Utc;
use realworld_rust_app::api::create_router;
use realworld_rust_app::application::services::article_service::ArticleService;
use realworld_rust_app::application::services::comment_service::CommentService;
use realworld_rust_app::application::services::profile_service::ProfileService;
use realworld_rust_app::application::services::user_service::UserService;
use realworld_rust_app::domain::repositories::ArticleRepository;
use realworld_rust_app::domain::repositories::CommentRepository;
use realworld_rust_app::domain::repositories::ProfileRepository;
use realworld_rust_app::domain::repositories::UserRepository;
use realworld_rust_app::infrastructure::db::repositories::article_repository::PostgresArticleRepository;
use realworld_rust_app::infrastructure::db::repositories::comment_repository::PostgresCommentRepository;
use realworld_rust_app::infrastructure::db::repositories::profile_repository::PostgresProfileRepository;
use realworld_rust_app::infrastructure::db::repositories::user_repository::PostgresUserRepository;
use reqwest::StatusCode;
use serde_json::Value;
use serde_json::json;
use std::sync::Arc;
use std::sync::Once;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use tower_http::trace::TraceLayer;

static INIT: Once = Once::new();

pub async fn setup_test_app() -> (TestServer, testcontainers::ContainerAsync<Postgres>) {
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("info,sqlx=debug")
            .with_test_writer()
            .try_init();

        jsonwebtoken::crypto::aws_lc::DEFAULT_PROVIDER
            .install_default()
            .expect("JsonWebToken provider failed to install");
    });

    let container = Postgres::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let db_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(15))
        .connect(&db_url)
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let user_repo: Arc<dyn UserRepository> = Arc::new(PostgresUserRepository::new(pool.clone()));
    let user_service = Arc::new(UserService::new(
        Arc::clone(&user_repo),
        "test-secret-key-12345".to_string(),
        24,
    ));

    let profile_repo: Arc<dyn ProfileRepository> =
        Arc::new(PostgresProfileRepository::new(pool.clone()));
    let profile_service = Arc::new(ProfileService::new(
        Arc::clone(&profile_repo),
        Arc::clone(&user_repo),
    ));

    let article_repo: Arc<dyn ArticleRepository> =
        Arc::new(PostgresArticleRepository::new(pool.clone()));
    let article_service = Arc::new(ArticleService::new(Arc::clone(&article_repo)));

    let comment_repo: Arc<dyn CommentRepository> =
        Arc::new(PostgresCommentRepository::new(pool.clone()));
    let comment_service = Arc::new(CommentService::new(
        Arc::clone(&comment_repo),
        Arc::clone(&article_repo),
    ));

    let app = create_router(
        user_service,
        profile_service,
        article_service,
        comment_service,
    );

    let server = TestServer::new(app);
    (server, container)
}

pub struct TestUser {
    pub username: String,
    pub email: String,
    pub token: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

impl TestUser {
    pub async fn new(server: &TestServer, prefix: &str) -> Self {
        let uid = uuid::Uuid::new_v4().to_string()[..8].to_string();
        let username = format!("{}_{}", prefix, uid);

        register_user(server, &username).await
    }
}

pub async fn register_user(server: &TestServer, username: &str) -> TestUser {
    let response = server
        .post("/api/users")
        .json(&json!({
            "user": {
                "username": username,
                "email": format!("{}@test.com", username),
                "password": "password123"
            }
        }))
        .await;

    response.assert_status(StatusCode::CREATED);

    let body: Value = response.json();
    let user_data = &body["user"];

    TestUser {
        username: user_data["username"].as_str().unwrap().to_string(),
        email: user_data["email"].as_str().unwrap().to_string(),
        token: user_data["token"].as_str().unwrap().to_string(),
        bio: user_data["bio"].as_str().map(|s| s.to_string()),
        image: user_data["image"].as_str().map(|s| s.to_string()),
    }
}

pub struct TestProfile {
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub following: bool,
}

pub struct TestArticle {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub tag_list: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub favorited: bool,
    pub favorites_count: u32,
    pub author: TestProfile,
}

impl TestArticle {
    pub async fn new(
        server: &TestServer,
        title: &str,
        description: &str,
        body: &str,
        tag_list: Vec<String>,
        token: &str,
    ) -> Self {
        create_article(server, title, description, body, tag_list, token).await
    }
}

async fn create_article(
    server: &TestServer,
    title: &str,
    description: &str,
    body: &str,
    tag_list: Vec<String>,
    token: &str,
) -> TestArticle {
    let response = server
        .post("/api/articles")
        .add_header("Authorization", format!("Token {}", token))
        .json(&json!({
            "article": {
                "title": title,
                "description": description,
                "body": body,
                "tagList": tag_list
            }
        }))
        .await;

    response.assert_status(StatusCode::CREATED);

    let body: Value = response.json();
    let article_data = &body["article"];

    TestArticle {
        slug: article_data["slug"].as_str().unwrap().to_string(),
        title: article_data["title"].as_str().unwrap().to_string(),
        description: article_data["description"].as_str().unwrap().to_string(),
        body: article_data["body"].as_str().unwrap().to_string(),
        tag_list: article_data["tagList"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect(),
        created_at: article_data["createdAt"]
            .as_str()
            .unwrap()
            .parse::<DateTime<Utc>>()
            .expect("Invalid createdAt format"),

        updated_at: article_data["updatedAt"]
            .as_str()
            .unwrap()
            .parse::<DateTime<Utc>>()
            .expect("Invalid updatedAt format"),
        favorited: article_data["favorited"].as_bool().unwrap_or(false),
        favorites_count: article_data["favoritesCount"].as_u64().unwrap_or(0) as u32,
        author: TestProfile {
            username: article_data["author"]["username"]
                .as_str()
                .unwrap()
                .to_string(),
            bio: article_data["author"]["bio"]
                .as_str()
                .map(|s| s.to_string()),
            image: article_data["author"]["image"]
                .as_str()
                .map(|s| s.to_string()),
            following: article_data["author"]["following"].as_bool().unwrap(),
        },
    }
}

pub struct TestComment {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub body: String,
    pub author: TestProfile,
}

impl TestComment {
    pub async fn new(server: &TestServer, slug: &str, body: &str, token: &str) -> Self {
        create_comment(server, slug, body, token).await
    }
}

async fn create_comment(server: &TestServer, slug: &str, body: &str, token: &str) -> TestComment {
    let response = server
        .post(&format!("/api/articles/{}/comments", slug))
        .add_header("Authorization", format!("Token {}", token))
        .json(&json!({
            "comment": {
                "body": body
            }
        }))
        .await;

    response.assert_status(StatusCode::CREATED);
    let body: Value = response.json();
    let c = &body["comment"];

    TestComment {
        id: c["id"].as_i64().expect("comment id is missing") as i32,
        body: c["body"].as_str().unwrap_or_default().to_string(),
        created_at: c["createdAt"]
            .as_str()
            .and_then(|s| s.parse::<DateTime<Utc>>().ok())
            .expect("invalid createdAt"),
        updated_at: c["updatedAt"]
            .as_str()
            .and_then(|s| s.parse::<DateTime<Utc>>().ok())
            .expect("invalid updatedAt"),
        author: TestProfile {
            username: c["author"]["username"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            bio: c["author"]["bio"].as_str().map(|s| s.to_string()),
            image: c["author"]["image"].as_str().map(|s| s.to_string()),
            following: c["author"]["following"].as_bool().unwrap_or(false),
        },
    }
}
