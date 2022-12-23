use std::sync::Arc;

use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};

use std::{fs::File, io::BufReader};

use crate::config::YummyConfig;

pub fn load_rustls_config(config: Arc<YummyConfig>) -> Option<rustls::ServerConfig> {
    let (cert_file_path, key_file_path) = match (config.tls_cert_path.as_ref(), config.tls_key_path.as_ref()) {
        (None, None) => return None,
        (Some(cert_file_path), Some(key_file_path)) => (cert_file_path, key_file_path),
        _ => {
            log::error!("Please specific TLS certificates 'cert', 'key' file path");
            return None;
        }
    };

    let (cert_file, key_file) = match (File::open(cert_file_path), File::open(key_file_path)) {
        (Ok(cert_file), Ok(key_file)) => (cert_file, key_file),
        (Err(error), _) => {
            log::error!("TLS cert file not exists. Error: {}", error);
            return None;
        },
        (_, Err(error)) => {
            log::error!("TLS cert file not exists. Error: {}", error);
            return None;
        }
    };

    // init server config builder with safe defaults
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();

    // load TLS key/cert files
    let cert_file = &mut BufReader::new(cert_file);
    let key_file = &mut BufReader::new(key_file);

    // convert files to key/cert objects
    let cert_chain = match certs(cert_file) {
        Ok(cert_chain) => cert_chain.into_iter().map(Certificate).collect(),
        Err(error) => {
            log::error!("Cert chain had error. Error: {}", error);
            return None;
        }
    };

    let mut keys: Vec<PrivateKey> = match pkcs8_private_keys(key_file) {
        Ok(keys) => keys.into_iter().map(PrivateKey).collect(),
        Err(error) => {
            log::error!("Pkcs8 private keys had error. Error: {}", error);
            return None;
        }
    };

    // exit if no keys could be parsed
    if keys.is_empty() {
        eprintln!("Could not locate PKCS 8 private keys.");
        return None;
    }

    config.with_single_cert(cert_chain, keys.remove(0)).ok()
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::sync::Arc;
    use std::io::{Write, Read, Seek, SeekFrom};

    use crate::config::get_configuration;

    use super::load_rustls_config;

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
}
