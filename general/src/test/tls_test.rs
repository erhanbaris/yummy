use std::sync::Arc;

use crate::config::get_configuration;
use crate::tls::{load_temporary_rustls_config, load_rustls_config};

#[test]
fn no_config() {
    let config = get_configuration();
    let rustls_config = load_rustls_config(config);

    assert!(rustls_config.is_none());
}

#[test]
fn invalid_cert() {
    let mut config = get_configuration().as_ref().clone();
    config.tls_cert_path = Some("dummy".to_string());
    let rustls_config = load_rustls_config(Arc::new(config));
    assert!(rustls_config.is_none());

    let mut config = get_configuration().as_ref().clone();
    config.tls_key_path = Some("dummy".to_string());
    let rustls_config = load_rustls_config(Arc::new(config));
    assert!(rustls_config.is_none());
}

#[test]
fn valid_cert() {
    let rustls_config = load_temporary_rustls_config(get_configuration());
    assert!(rustls_config.is_some());
}
