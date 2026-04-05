use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub server_port: u16,

    pub database_url: String,
    pub db_max_connections: u32,
    pub db_min_connections: u32,
    pub db_acquire_timeout_sec: u64,
    pub db_idle_timeout_sec: u64,

    pub jwt_secret: String,
    pub jwt_exp_hours: u64,

    pub rust_log: String,
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenv().ok();

        envy::from_env::<Config>()
    }
}
