use std::time::Duration;
use actix::Recipient;
use general::config::YummyConfig;
use general::auth::validate_auth;
use general::config::configure_environment;
use general::state::SendMessage;
use general::state::YummyState;

#[cfg(feature = "stateless")]
use general::test::cleanup_redis;
use std::sync::Arc;

use actix::Actor;
use actix::Addr;
use anyhow::Ok;
use database::{create_database, create_connection};

use crate::api::conn::CommunicationManager;

use super::AuthManager;
use super::*;

use general::test::DummyClient;

fn create_actor(config: Arc<YummyConfig>) -> anyhow::Result<(Addr<AuthManager<database::SqliteStore>>, Arc<DummyClient>)> {
    let connection = create_connection(":memory:")?;
    #[cfg(feature = "stateless")]
    let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();

    #[cfg(feature = "stateless")]
    cleanup_redis(conn.clone());

    CommunicationManager::new(config.clone()).start();
    let states = YummyState::new(config.clone(), #[cfg(feature = "stateless")] conn);

    create_database(&mut connection.clone().get()?)?;
    Ok((AuthManager::<database::SqliteStore>::new(config.clone(), states, Arc::new(connection)).start(), Arc::new(DummyClient::default())))
}

/* email unit tests */
#[actix::test]
async fn create_user_via_email() -> anyhow::Result<()> {
    let (address, socket) = create_actor(::general::config::get_configuration())?;
    address.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: true,
        socket
    }).await??;
    Ok(())
}

#[actix::test]
async fn login_user_via_email() -> anyhow::Result<()> {
    let mut config = ::general::config::get_configuration().deref().clone();
    config.connection_restore_wait_timeout = Duration::from_secs(1);
    config.heartbeat_interval = Duration::from_secs(1);
    config.client_timeout = Duration::from_secs(1);
    
    let (address, socket) = create_actor(Arc::new(config))?;
    address.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: true,
        socket: socket.clone()
    }).await??;

    let auth = socket.clone().auth.lock().unwrap().clone();
    address.send(StartUserTimeout {
        session_id: auth.session.clone(),
        user_id: auth.id
    }).await??;

    actix::clock::sleep(std::time::Duration::new(3, 0)).await;

    address.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: false,
        socket
    }).await??;

    return Ok(());
}

#[actix::test]
async fn failed_login_user_via_email_1() -> anyhow::Result<()> {
    let (address, socket) = create_actor(::general::config::get_configuration())?;
    let result = address.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: false,
        socket
    }).await?;

    assert_eq!(result.unwrap_err().to_string(), "Email and/or password not valid".to_string());
    Ok(())
}

#[actix::test]
async fn failed_login_user_via_email_2() -> anyhow::Result<()> {
    let (address, socket) = create_actor(::general::config::get_configuration())?;
    address.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: true,
        socket: socket.clone()
    }).await??;

    let result = address.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password: "wrong password".into(),
        if_not_exist_create: true,
        socket: socket.clone()
    }).await?;

    assert_eq!(result.unwrap_err().to_string(), "Email and/or password not valid".to_string());
    Ok(())
}

/* device id unit tests */
#[actix::test]
async fn create_user_via_device_id() -> anyhow::Result<()> {
    let (address, socket) = create_actor(::general::config::get_configuration())?;
    address.send(DeviceIdAuthRequest::new("1234567890".to_string(), socket)).await??;
    Ok(())
}

#[actix::test]
async fn login_user_via_device_id() -> anyhow::Result<()> {
    let mut config = ::general::config::get_configuration().deref().clone();
    config.connection_restore_wait_timeout = Duration::from_secs(1);
    config.heartbeat_interval = Duration::from_secs(1);
    config.client_timeout = Duration::from_secs(1);
    
    let (address, socket) = create_actor(Arc::new(config))?;
    address.send(DeviceIdAuthRequest::new("1234567890".to_string(), socket.clone())).await??;

    let created_token = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    let auth = socket.clone().auth.lock().unwrap().clone();
    
    address.send(StartUserTimeout {
        session_id: auth.session.clone(),
        user_id: auth.id
    }).await??;

    actix::clock::sleep(std::time::Duration::new(3, 0)).await;

    address.send(DeviceIdAuthRequest::new("1234567890".to_string(), socket.clone())).await??;
    let logged_in_token = socket.clone().messages.lock().unwrap().pop_back().unwrap();

    assert_ne!(created_token, logged_in_token);

    return Ok(());
}

#[actix::test]
async fn login_users_via_device_id() -> anyhow::Result<()> {
    let (address, socket) = create_actor(::general::config::get_configuration())?;
    address.send(DeviceIdAuthRequest::new("1234567890".to_string(), socket.clone())).await??;
    let login_1 = socket.clone().auth.lock().unwrap().clone();

    address.send(DeviceIdAuthRequest::new("abcdef".to_string(), socket.clone())).await??;
    let login_2 = socket.clone().auth.lock().unwrap().clone();
    assert_ne!(login_1, login_2);

    Ok(())
}

/* custom id unit tests */
#[actix::test]
async fn create_user_via_custom_id() -> anyhow::Result<()> {
    let (address, socket) = create_actor(::general::config::get_configuration())?;
    address.send(CustomIdAuthRequest::new("1234567890".to_string(), socket)).await??;
    Ok(())
}

#[actix::test]
async fn login_user_via_custom_id() -> anyhow::Result<()> {
    let mut config = ::general::config::get_configuration().deref().clone();
    config.connection_restore_wait_timeout = Duration::from_secs(1);
    config.heartbeat_interval = Duration::from_secs(1);
    config.client_timeout = Duration::from_secs(1);
    
    let (address, socket) = create_actor(Arc::new(config))?;
    address.send(CustomIdAuthRequest::new("1234567890".to_string(), socket.clone())).await??;

    let created_token = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    let auth = socket.clone().auth.lock().unwrap().clone();

    address.send(StartUserTimeout {
        session_id: auth.session.clone(),
        user_id: auth.id
    }).await??;

    actix::clock::sleep(std::time::Duration::new(3, 0)).await;

    address.send(CustomIdAuthRequest::new("1234567890".to_string(), socket.clone())).await??;
    let logged_in_token = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    assert_ne!(created_token, logged_in_token);

    return Ok(());
}

#[actix::test]
async fn login_users_via_custom_id() -> anyhow::Result<()> {
    let (address, socket) = create_actor(::general::config::get_configuration())?;
    address.send(CustomIdAuthRequest::new("1234567890".to_string(), socket.clone())).await??;
    let login_1 = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    
    address.send(CustomIdAuthRequest::new("abcdef".to_string(), socket.clone())).await??;
    let login_2 = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    assert_ne!(login_1, login_2);

    Ok(())
}

/* restore token unit tests */
#[actix::test]
async fn token_restore_test_1() -> anyhow::Result<()> {
    configure_environment();
    let config = ::general::config::get_configuration();
    let (address, socket) = create_actor(config.clone())?;
    address.send(DeviceIdAuthRequest::new("1234567890".to_string(), socket.clone())).await??;
    let old_token: GenericAnswer<String> = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let old_token = old_token.result.unwrap_or_default();

    // Wait 1 second
    actix::clock::sleep(std::time::Duration::new(1, 0)).await;
    address.send(RestoreTokenRequest { token: old_token.to_string(), socket: socket.clone() }).await??;
    let new_token: GenericAnswer<String> = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let new_token = new_token.result.unwrap_or_default();
    
    assert_ne!(old_token.clone(), new_token.clone());

    let old_claims =  validate_auth(config.clone(), old_token).unwrap();
    let new_claims =  validate_auth(config.clone(), new_token).unwrap();

    assert_eq!(old_claims.user.id.clone(), new_claims.user.id.clone());
    assert_eq!(old_claims.user.name.clone(), new_claims.user.name.clone());
    assert_eq!(old_claims.user.email.clone(), new_claims.user.email.clone());
    assert_eq!(old_claims.user.session.clone(), new_claims.user.session.clone());

    assert!(old_claims.exp < new_claims.exp);

    Ok(())
}

#[actix::test]
async fn fail_token_restore_test_1() -> anyhow::Result<()> {
    let mut config = ::general::config::get_configuration().deref().clone();
    config.token_lifetime = Duration::from_secs(1);

    let (address, socket) = create_actor(Arc::new(config))?;
    address.send(DeviceIdAuthRequest::new("1234567890".to_string(), socket.clone())).await??;
    
    let old_token = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    let old_token: GenericAnswer<String> = old_token.into();
    let old_token = old_token.result.unwrap_or_default();

    // Wait 3 seconds
    actix::clock::sleep(std::time::Duration::new(3, 0)).await;
    assert!(address.send(RestoreTokenRequest { token: old_token.to_string(), socket: socket.clone() }).await?.is_err());
    let message = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    assert!(message.contains("User token is not valid"));
    Ok(())
}

/* refreh token unit tests */
#[actix::test]
async fn token_refresh_test_1() -> anyhow::Result<()> {
    configure_environment();
    let config = ::general::config::get_configuration();
    let (address, socket) = create_actor(config.clone())?;
    address.send(DeviceIdAuthRequest::new("1234567890".to_string(), socket.clone())).await??;
    let old_token = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    let old_token: GenericAnswer<String> = old_token.into();
    let old_token = old_token.result.unwrap_or_default();

    // Wait 1 second
    actix::clock::sleep(std::time::Duration::new(1, 0)).await;
    address.send(RefreshTokenRequest { token: old_token.to_string(), socket: socket.clone() }).await??;

    let new_token = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    let new_token: GenericAnswer<String> = new_token.into();
    let new_token = new_token.result.unwrap_or_default();

    assert_ne!(old_token.clone(), new_token.clone());

    let old_claims =  validate_auth(config.clone(), old_token).unwrap();
    let new_claims =  validate_auth(config.clone(), new_token).unwrap();

    assert_eq!(old_claims.user.id.clone(), new_claims.user.id.clone());
    assert_eq!(old_claims.user.name.clone(), new_claims.user.name.clone());
    assert_eq!(old_claims.user.email.clone(), new_claims.user.email.clone());
    assert_eq!(old_claims.user.session.clone(), new_claims.user.session.clone());

    assert!(old_claims.exp < new_claims.exp);

    Ok(())
}

#[actix::test]
async fn token_refresh_test_2() -> anyhow::Result<()> {
    configure_environment();
    let config = ::general::config::get_configuration();
    let (address, socket) = create_actor(config.clone())?;
    address.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: true, socket: socket.clone()
    }).await??;


    let old_token = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    let old_token: GenericAnswer<String> = old_token.into();
    let old_token = old_token.result.unwrap_or_default();
    
    // Wait 1 second
    actix::clock::sleep(std::time::Duration::new(1, 0)).await;
    address.send(RefreshTokenRequest{ token: old_token.clone(), socket: socket.clone() }).await??;
    let new_token = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    let new_token: GenericAnswer<String> = new_token.into();
    let new_token = new_token.result.unwrap_or_default();
    
    assert_ne!(old_token.clone(), new_token.clone());

    let old_claims =  validate_auth(config.clone(), old_token).unwrap();
    let new_claims =  validate_auth(config.clone(), new_token).unwrap();

    assert_eq!(old_claims.user.id.clone(), new_claims.user.id.clone());
    assert_eq!(old_claims.user.name.clone(), new_claims.user.name.clone());
    assert_eq!(old_claims.user.email.clone(), new_claims.user.email.clone());
    assert_eq!(old_claims.user.session.clone(), new_claims.user.session.clone());

    assert!(old_claims.exp < new_claims.exp);

    Ok(())
}