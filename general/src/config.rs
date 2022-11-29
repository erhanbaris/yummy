use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use std::env;

pub const DEFAULT_CLIENT_TIMEOUT: u64 = 20; // in seconds
pub const DEFAULT_CONNECTION_RESTORE_WAIT_TIMEOUT: u64 = 10; // in seconds
pub const DEFAULT_HEARTBEAT_INTERVAL: u64 = 10; // in seconds
pub const DEFAULT_TOKEN_LIFETIME: u64 = 24 * 60 * 60; // in seconds
pub const DEFAULT_API_KEY_NAME: &str = "x-yummy-api";
pub const DEFAULT_USER_AUTH_KEY_NAME: &str = "x-yummy-auth";
pub const DEFAULT_SALT_KEY: &str = "YUMMY-SALT";
pub const DEFAULT_DATABASE_URL: &str = "yummy.db";
pub const DEFAULT_DEFAULT_INTEGRATION_KEY: &str = "YummyYummy";

#[derive(Debug, Default, Clone)]
pub struct YummyConfig {
    pub token_lifetime: Duration,
    pub heartbeat_interval: Duration,
    pub client_timeout: Duration,
    pub connection_restore_wait_timeout: Duration,

    pub integration_key: String,
    pub api_key_name: String,
    pub user_auth_key_name: String,
    pub salt_key: String,
    pub database_url: String,
}

pub fn get_env_var<R: Clone + FromStr>(key: &str, default: R) -> R {
    env::var(key)
        .map(|value| value.parse::<R>().unwrap_or_else(|_| default.clone()))
        .unwrap_or(default)
}

pub fn get_configuration() -> Arc<YummyConfig> {
    let mut yummy_config = YummyConfig::default();
    yummy_config.client_timeout = Duration::from_secs(get_env_var("CLIENT_TIMEOUT", DEFAULT_CLIENT_TIMEOUT));
    yummy_config.connection_restore_wait_timeout = Duration::from_secs(get_env_var("CONNECTION_RESTORE_WAIT_TIMEOUT", DEFAULT_CONNECTION_RESTORE_WAIT_TIMEOUT));
    yummy_config.heartbeat_interval = Duration::from_secs(get_env_var("HEARTBEAT_INTERVAL", DEFAULT_HEARTBEAT_INTERVAL));
    yummy_config.token_lifetime = Duration::from_secs(get_env_var("TOKEN_LIFETIME", DEFAULT_TOKEN_LIFETIME));
    yummy_config.api_key_name = get_env_var("API_KEY_NAME", DEFAULT_API_KEY_NAME.to_string());
    yummy_config.user_auth_key_name = get_env_var("USER_AUTH_KEY_NAME", DEFAULT_USER_AUTH_KEY_NAME.to_string());
    yummy_config.salt_key = get_env_var("SALT_KEY", DEFAULT_SALT_KEY.to_string());
    yummy_config.integration_key = get_env_var("INTEGRATION_KEY", DEFAULT_DEFAULT_INTEGRATION_KEY.to_string());
    yummy_config.database_url = get_env_var("DATABASE_URL", DEFAULT_DATABASE_URL.to_string());
    Arc::new(yummy_config)
}
