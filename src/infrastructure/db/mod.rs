pub mod repositories;

use crate::Config;
use sqlx::ConnectOptions;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::{str::FromStr, time::Duration};
use tracing::log::LevelFilter;

pub async fn init_pool(config: &Config) -> Result<PgPool, sqlx::Error> {
    let connect_options = PgConnectOptions::from_str(&config.database_url)?
        .log_statements(LevelFilter::Debug)
        .log_slow_statements(LevelFilter::Warn, Duration::from_millis(100));

    PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .min_connections(config.db_min_connections)
        .acquire_timeout(Duration::from_secs(config.db_acquire_timeout_sec))
        .idle_timeout(Duration::from_secs(config.db_idle_timeout_sec))
        .connect_with(connect_options)
        .await
}
