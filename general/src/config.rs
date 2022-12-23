use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use std::env;

use rand::{distributions::Alphanumeric, Rng};

pub const DEFAULT_BIND_IP: &str = "0.0.0.0";
pub const DEFAULT_BIND_PORT: usize = 9090;
pub const DEFAULT_RUST_LOG: &str = "debug,backend,actix_web=debug";

pub const DEFAULT_HEARTBEAT_TIMEOUT: u64 = 20; // in seconds
pub const DEFAULT_CONNECTION_RESTORE_WAIT_TIMEOUT: u64 = 10; // in seconds
pub const DEFAULT_HEARTBEAT_INTERVAL: u64 = 10; // in seconds
pub const DEFAULT_TOKEN_LIFETIME: u64 = 24 * 60 * 60; // in seconds
pub const DEFAULT_MAX_USER_META: usize = 10;
pub const DEFAULT_API_KEY_NAME: &str = "x-yummy-api";
pub const DEFAULT_SALT_KEY: &str = "YUMMY-SALT";
pub const DEFAULT_DATABASE_PATH: &str = "yummy.db";
pub const DEFAULT_DEFAULT_INTEGRATION_KEY: &str = "YummyYummy";
pub const DEFAULT_ROOM_PASSWORD_CHARSET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
pub const DEFAULT_ROOM_PASSWORD_LENGTH: usize = 4;

#[cfg(feature = "stateless")]
pub const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1/";

#[cfg(feature = "stateless")]
pub const DEFAULT_REDIS_PREFIX: &str = "";


#[derive(Debug, Default, Clone)]
pub struct YummyConfig {
    pub server_name: String,
    pub bind_ip: String,
    pub bind_port: String,

    pub rust_log: String,
    
    pub token_lifetime: Duration,
    pub heartbeat_interval: Duration,
    pub heartbeat_timeout: Duration,
    pub connection_restore_wait_timeout: Duration,

    pub max_user_meta: usize,
    pub room_password_charset: Vec<u8>,
    pub room_password_length: usize,

    pub integration_key: String,
    pub api_key_name: String,
    pub salt_key: String,
    pub database_path: String,

    #[cfg(feature = "stateless")]
    pub redis_url: String,

    #[cfg(feature = "stateless")]
    pub redis_prefix: String
}

pub fn get_env_var<R: Clone + FromStr>(key: &str, default: R) -> R {
    env::var(key)
        .map(|value| value.parse::<R>().unwrap_or_else(|_| default.clone()))
        .unwrap_or(default)
}

pub fn configure_environment() {
    let profile = get_profile();
    dotenv::from_filename(format!("{}.env", profile)).ok();
    dotenv::dotenv().ok();
}

pub fn get_profile() -> &'static str {
    if cfg!(test) {
        "test"
    } else if cfg!(debug_assertions) {
        "dev"
    } else {
        "prod"
    }
}

pub fn get_configuration() -> Arc<YummyConfig> {
    let server_name: String = format!("YUMMY-{}", rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect::<String>());

    Arc::new(YummyConfig {
        server_name: get_env_var("SERVER_NAME", server_name),
        bind_ip: get_env_var("BIND_IP", DEFAULT_BIND_IP.to_string()),
        bind_port: get_env_var("BIND_PORT", DEFAULT_BIND_PORT.to_string()),
        rust_log: get_env_var("RUST_LOG", DEFAULT_RUST_LOG.to_string()),
        heartbeat_timeout: Duration::from_secs(get_env_var("HEARTBEAT_TIMEOUT", DEFAULT_HEARTBEAT_TIMEOUT)),
        connection_restore_wait_timeout: Duration::from_secs(get_env_var("CONNECTION_RESTORE_WAIT_TIMEOUT", DEFAULT_CONNECTION_RESTORE_WAIT_TIMEOUT)),
        heartbeat_interval: Duration::from_secs(get_env_var("HEARTBEAT_INTERVAL", DEFAULT_HEARTBEAT_INTERVAL)),
        token_lifetime: Duration::from_secs(get_env_var("TOKEN_LIFETIME", DEFAULT_TOKEN_LIFETIME)),
        api_key_name: get_env_var("API_KEY_NAME", DEFAULT_API_KEY_NAME.to_string()),
        salt_key: get_env_var("SALT_KEY", DEFAULT_SALT_KEY.to_string()),
        integration_key: get_env_var("INTEGRATION_KEY", DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()),
        database_path: get_env_var("DATABASE_PATH", DEFAULT_DATABASE_PATH.to_string()),
        max_user_meta: get_env_var("MAX_USER_META", DEFAULT_MAX_USER_META),
        room_password_charset: get_env_var("ROOM_PASSWORD_CHARSET", DEFAULT_ROOM_PASSWORD_CHARSET.to_string()).as_bytes().to_vec(),
        room_password_length: get_env_var("ROOM_PASSWORD_LENGTH", DEFAULT_ROOM_PASSWORD_LENGTH),

        #[cfg(feature = "stateless")] redis_url: get_env_var("REDIS_URL", DEFAULT_REDIS_URL.to_string()),
        #[cfg(feature = "stateless")] redis_prefix: get_env_var("REDIS_PREFIX", DEFAULT_REDIS_PREFIX.to_string()),
    })
}
