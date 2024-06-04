use serde::Deserialize;

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Config {
    pub server_address: String,
    pub cache_ttl: usize,
    pub postgres: deadpool_postgres::Config,
    pub redis: deadpool_redis::Config,
}
