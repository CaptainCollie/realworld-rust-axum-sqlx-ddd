pub mod repositories;

use crate::Config;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub async fn init_pool(config: &Config) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .min_connections(config.db_min_connections)
        .acquire_timeout(Duration::from_secs(config.db_acquire_timeout_sec))
        .idle_timeout(Duration::from_secs(config.db_idle_timeout_sec))
        .connect(&config.database_url)
        .await
}
