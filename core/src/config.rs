use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use std::env;

pub const DEFAULT_CLIENT_TIMEOUT: u64 = 20; // in seconds
pub const DEFAULT_HEARTBEAT_INTERVAL: u64 = 10; // in seconds
pub const DEFAULT_TOKEN_LIFETIME: i64 = 24; // in seconds
pub const DEFAULT_COOKIE_KEY: &str = "x-yummy-auth";
pub const DEFAULT_SALT_KEY: &str = "YUMMY-SALT";
pub const DEFAULT_DATABASE_URL: &str = "yummy.db";

#[derive(Debug, Default, Clone)]
pub struct YummyConfig {
    pub token_lifetime: i64,
    pub cookie_key: String,
    pub heartbeat_interval: Duration,
    pub client_timeout: Duration,
    pub salt_key: String,
    pub database_url: String,
    pub hasher: harsh::Harsh
}

pub fn get_env_var<R: Clone + FromStr>(key: &str, default: R) -> R {
    env::var(key)
        .map(|value| value.parse::<R>().unwrap_or(default.clone()))
        .unwrap_or(default)
}

pub fn get_configuration() -> Arc<YummyConfig> {
    let mut yummy_config = YummyConfig::default();
    yummy_config.client_timeout = Duration::from_secs(get_env_var("CLIENT_TIMEOUT", DEFAULT_CLIENT_TIMEOUT));
    yummy_config.heartbeat_interval = Duration::from_secs(get_env_var("HEARTBEAT_INTERVAL", DEFAULT_HEARTBEAT_INTERVAL));
    yummy_config.token_lifetime = get_env_var("TOKEN_LIFETIME", DEFAULT_TOKEN_LIFETIME);
    yummy_config.cookie_key = get_env_var("COOKIE_KEY", DEFAULT_COOKIE_KEY.to_string());
    yummy_config.salt_key = get_env_var("SALT_KEY", DEFAULT_SALT_KEY.to_string());
    yummy_config.hasher = harsh::Harsh::builder().salt(&yummy_config.salt_key[..]).length(5).build().unwrap();
    yummy_config.database_url = get_env_var("DATABASE_URL", DEFAULT_DATABASE_URL.to_string());
    Arc::new(yummy_config)
}
