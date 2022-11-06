use lazy_static::lazy_static;
use std::time::Duration;

pub static TOKEN_LIFETIME: i64 = 24;
pub static COOKIE_KEY: &str = "x-auth";
pub static HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);
pub static CLIENT_TIMEOUT: Duration = Duration::from_secs(20);

#[cfg(any(feature="static-salt", feature="test"))]
pub static SALT: &'static str = "erhan baris";

#[cfg(not(any(feature="static-salt", feature="test")))]
lazy_static! {
    pub static ref SALT: String = {
        use rand::{thread_rng, Rng};
        use rand::distributions::Alphanumeric;
        let m: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();
        m
    };
}

lazy_static! {
    pub static ref HASHER: harsh::Harsh = harsh::Harsh::builder().salt(&SALT[..]).length(5).build().unwrap();
}
