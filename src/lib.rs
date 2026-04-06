pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use infrastructure::config::Config;
use infrastructure::db::repositories::user_repository::PostgresUserRepository;

use application::services::user_service::UserService;
use infrastructure::db::init_pool;
use metrics_exporter_prometheus::PrometheusBuilder;
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt};

use crate::application::services::article_service::ArticleService;
use crate::application::services::comment_service::CommentService;
use crate::application::services::profile_service::ProfileService;
use crate::domain::repositories::{
    ArticleRepository, CommentRepository, ProfileRepository, UserRepository,
};
use crate::infrastructure::db::repositories::article_repository::PostgresArticleRepository;
use crate::infrastructure::db::repositories::comment_repository::PostgresCommentRepository;
use crate::infrastructure::db::repositories::profile_repository::PostgresProfileRepository;
use tracing_subscriber::prelude::*;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env().expect("Failed to load configuration");

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.rust_log));

    let registry = tracing_subscriber::registry().with(filter);

    if config.is_docker {
        registry.with(fmt::layer().json()).init();
    } else {
        registry.with(fmt::layer().with_target(false)).init();
    }

    let recorder_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install prometheus recorder");

    jsonwebtoken::crypto::aws_lc::DEFAULT_PROVIDER
        .install_default()
        .expect("JsonWebToken provider failed to install");

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
        recorder_handle,
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("🚀 Server started at http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
