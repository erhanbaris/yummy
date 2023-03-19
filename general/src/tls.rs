use std::sync::Arc;

use model::config::YummyConfig;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};

use std::{fs::File, io::BufReader};

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
        log::error!("Could not locate PKCS 8 private keys.");
        return None;
    }

    match config.with_single_cert(cert_chain, keys.remove(0)) {
        Ok(config) => Some(config),
        Err(error) => {
            log::error!("Could not combine cert. Error: {}", error);
            None
        }
    }
}

pub fn load_temporary_rustls_config(config: Arc<YummyConfig>) -> Option<rustls::ServerConfig> {
    use tempfile::tempdir;
    use rcgen::generate_simple_self_signed;
    use std::io::Write;

    let subject_alt_names = vec!["127.0.0.1".to_string(), "localhost".to_string()];

    let cert = generate_simple_self_signed(subject_alt_names).ok()?;

    let cert_file = cert.serialize_pem().ok()?;
    let key_file = cert.serialize_private_key_pem();
    
    let dir = tempdir().ok()?;

    let cert_file_path = dir.path().join("cert_file.txt");
    let key_file_path = dir.path().join("key_file.txt");
    
    let mut file = File::create(cert_file_path.clone()).ok()?;
    writeln!(file, "{}", cert_file).ok()?;
    let cert_file_path = cert_file_path.to_str()?.to_string();

    let mut file = File::create(key_file_path.clone()).ok()?;
    writeln!(file, "{}", key_file).ok()?;
    let key_file_path = key_file_path.to_str()?.to_string();

    let mut config = config.as_ref().clone();
    config.tls_cert_path = Some(cert_file_path);
    config.tls_key_path = Some(key_file_path);

    load_rustls_config(Arc::new(config))
}
