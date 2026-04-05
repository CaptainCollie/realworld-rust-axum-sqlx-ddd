pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::Body;
use axum::http::Request;
use infrastructure::config::Config;
use infrastructure::db::repositories::user_repository::PostgresUserRepository;

use application::services::user_service::UserService;
use infrastructure::db::init_pool;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info_span;

use crate::application::services::article_service::ArticleService;
use crate::application::services::comment_service::CommentService;
use crate::application::services::profile_service::ProfileService;
use crate::domain::repositories::{
    ArticleRepository, CommentRepository, ProfileRepository, UserRepository,
};
use crate::infrastructure::db::repositories::article_repository::PostgresArticleRepository;
use crate::infrastructure::db::repositories::comment_repository::PostgresCommentRepository;
use crate::infrastructure::db::repositories::profile_repository::PostgresProfileRepository;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .init();

    jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER
        .install_default()
        .expect("JsonWebToken provider failed to install");

    let config = Config::from_env().expect("Failed to load configuration");

    let pool = init_pool(&config).await?;

    if env::var("RUN_MIGRATIONS").unwrap_or_default() == "true" {
        sqlx::migrate!("./migrations").run(&pool).await?;
        println!("✅ Database connected and migrations applied");
    }

    let user_repo: Arc<dyn UserRepository> = Arc::new(PostgresUserRepository::new(pool.clone()));
    let user_service = Arc::new(UserService::new(
        Arc::clone(&user_repo),
        config.jwt_secret,
        config.jwt_exp_hours,
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

    let app = api::create_router(
        user_service,
        profile_service,
        article_service,
        comment_service,
    )
    .layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request<Body>| {
                info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                )
            })
            .on_request(|request: &Request<Body>, _span: &tracing::Span| {
                tracing::info!(
                    "--> Started {} {} {:?}",
                    request.method(),
                    request.uri(),
                    request.body()
                );
            })
            .on_response(
                |response: &axum::http::Response<Body>,
                 latency: std::time::Duration,
                 _span: &tracing::Span| {
                    tracing::info!(
                        "<-- Finished with {} in {:?}; {:?}",
                        response.status(),
                        latency,
                        response.body()
                    );
                },
            ),
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("🚀 Server started at http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
