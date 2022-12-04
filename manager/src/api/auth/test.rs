use std::time::Duration;
use general::config::YummyConfig;
use general::auth::validate_auth;
use general::model::YummyState;
use std::sync::Arc;

use actix::Actor;
use actix::Addr;
use anyhow::Ok;
use database::{create_database, create_connection};

use super::AuthManager;
use super::*;

use crate::response::Response;

fn create_actor(config: Arc<YummyConfig>) -> anyhow::Result<Addr<AuthManager<database::SqliteStore>>> {
    let connection = create_connection(":memory:")?;
    let states = Arc::new(YummyState::default());
    create_database(&mut connection.clone().get()?)?;
    Ok(AuthManager::<database::SqliteStore>::new(config.clone(), states, Arc::new(connection)).start())
}

/* email unit tests */
#[actix::test]
async fn create_user_via_email() -> anyhow::Result<()> {
    let address = create_actor(::general::config::get_configuration())?;
    address.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password: "erhan".to_string(),
        if_not_exist_create: true
    }).await??;
    Ok(())
}

#[actix::test]
async fn login_user_via_email() -> anyhow::Result<()> {
    let mut config = ::general::config::get_configuration().deref().clone();
    config.connection_restore_wait_timeout = Duration::from_secs(1);
    config.heartbeat_interval = Duration::from_secs(1);
    config.client_timeout = Duration::from_secs(1);
    
    let address = create_actor(Arc::new(config))?;
    let response = address.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password: "erhan".to_string(),
        if_not_exist_create: true
    }).await??;

    if let Response::Auth(_, auth) = response.clone() {
        address.send(StartUserTimeout {
            session_id: auth.session.clone()
        }).await??;

        actix::clock::sleep(std::time::Duration::new(3, 0)).await;

        address.send(EmailAuthRequest {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: false
        }).await??;

        return Ok(());
    }

    return Err(anyhow::anyhow!("Unexpected response"));
}

#[actix::test]
async fn failed_login_user_via_email_1() -> anyhow::Result<()> {
    let address = create_actor(::general::config::get_configuration())?;
    let result = address.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password: "erhan".to_string(),
        if_not_exist_create: false
    }).await?;

    assert_eq!(result.unwrap_err().to_string(), "Email and/or password not valid".to_string());
    Ok(())
}

#[actix::test]
async fn failed_login_user_via_email_2() -> anyhow::Result<()> {
    let address = create_actor(::general::config::get_configuration())?;
    address.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password: "erhan".to_string(),
        if_not_exist_create: true
    }).await??;

    let result = address.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password: "wrong password".to_string(),
        if_not_exist_create: true
    }).await?;

    assert_eq!(result.unwrap_err().to_string(), "Email and/or password not valid".to_string());
    Ok(())
}

/* device id unit tests */
#[actix::test]
async fn create_user_via_device_id() -> anyhow::Result<()> {
    let address = create_actor(::general::config::get_configuration())?;
    address.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
    Ok(())
}

#[actix::test]
async fn login_user_via_device_id() -> anyhow::Result<()> {
    let mut config = ::general::config::get_configuration().deref().clone();
    config.connection_restore_wait_timeout = Duration::from_secs(1);
    config.heartbeat_interval = Duration::from_secs(1);
    config.client_timeout = Duration::from_secs(1);
    
    let address = create_actor(Arc::new(config))?;
    let created_token = address.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;

    if let Response::Auth(_, auth) = created_token.clone() {
        address.send(StartUserTimeout {
            session_id: auth.session.clone()
        }).await??;

        actix::clock::sleep(std::time::Duration::new(3, 0)).await;

        let logged_in_token = address.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
        assert_ne!(created_token, logged_in_token);

        return Ok(());
    }

    return Err(anyhow::anyhow!("Unexpected response"));
}

#[actix::test]
async fn login_users_via_device_id() -> anyhow::Result<()> {
    let address = create_actor(::general::config::get_configuration())?;
    let login_1 = address.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
    let login_2 = address.send(DeviceIdAuthRequest::new("abcdef".to_string())).await??;
    assert_ne!(login_1, login_2);

    Ok(())
}

/* custom id unit tests */
#[actix::test]
async fn create_user_via_custom_id() -> anyhow::Result<()> {
    let address = create_actor(::general::config::get_configuration())?;
    address.send(CustomIdAuthRequest::new("1234567890".to_string())).await??;
    Ok(())
}

#[actix::test]
async fn login_user_via_custom_id() -> anyhow::Result<()> {
    let mut config = ::general::config::get_configuration().deref().clone();
    config.connection_restore_wait_timeout = Duration::from_secs(1);
    config.heartbeat_interval = Duration::from_secs(1);
    config.client_timeout = Duration::from_secs(1);
    
    let address = create_actor(Arc::new(config))?;
    let created_token = address.send(CustomIdAuthRequest::new("1234567890".to_string())).await??;

    if let Response::Auth(_, auth) = created_token.clone() {
        address.send(StartUserTimeout {
            session_id: auth.session.clone()
        }).await??;

        actix::clock::sleep(std::time::Duration::new(3, 0)).await;

        let logged_in_token = address.send(CustomIdAuthRequest::new("1234567890".to_string())).await??;
        assert_ne!(created_token, logged_in_token);

        return Ok(());
    }

    return Err(anyhow::anyhow!("Unexpected response"));
}

#[actix::test]
async fn login_users_via_custom_id() -> anyhow::Result<()> {
    let address = create_actor(::general::config::get_configuration())?;
    let login_1 = address.send(CustomIdAuthRequest::new("1234567890".to_string())).await??;
    let login_2 = address.send(CustomIdAuthRequest::new("abcdef".to_string())).await??;
    assert_ne!(login_1, login_2);

    Ok(())
}

/* restore token unit tests */
#[actix::test]
async fn token_restore_test_1() -> anyhow::Result<()> {
    let config = ::general::config::get_configuration();
    let address = create_actor(config.clone())?;
    let old_token: Response = address.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
    let old_token = match old_token {
        Response::Auth(token, _) => token,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
    };

    // Wait 1 second
    actix::clock::sleep(std::time::Duration::new(1, 0)).await;
    let new_token: Response = address.send(RestoreTokenRequest { token: old_token.to_string() }).await??;
    let new_token = match new_token {
        Response::Auth(token, _) => token,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
    };
    
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

    let address = create_actor(Arc::new(config))?;
    let old_token: Response = address.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
    let old_token = match old_token {
        Response::Auth(token, _) => token,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
    };

    // Wait 3 seconds
    actix::clock::sleep(std::time::Duration::new(3, 0)).await;
    let response = address.send(RestoreTokenRequest { token: old_token.to_string() }).await?;
    
    if response.is_ok() {
        assert!(false, "Expected exception, received: {:?}", response);
    }
    
    Ok(())
}

/* refreh token unit tests */
#[actix::test]
async fn token_refresh_test_1() -> anyhow::Result<()> {
    let config = ::general::config::get_configuration();
    let address = create_actor(config.clone())?;
    let old_token: Response = address.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
    let old_token = match old_token {
        Response::Auth(token, _) => token,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
    };

    // Wait 1 second
    actix::clock::sleep(std::time::Duration::new(1, 0)).await;
    let new_token: Response = address.send(RefreshTokenRequest { token: old_token.to_string() }).await??;
    let new_token = match new_token {
        Response::Auth(token, _) => token,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
    };
    
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
    let config = ::general::config::get_configuration();
    let address = create_actor(config.clone())?;
    let old_token: Response = address.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password: "erhan".to_string(),
        if_not_exist_create: true
    }).await??;

    let old_token = match old_token {
        Response::Auth(token, _) => token,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
    };

    // Wait 1 second
    actix::clock::sleep(std::time::Duration::new(1, 0)).await;
    let new_token: Response = address.send(RefreshTokenRequest{ token: old_token.clone() }).await??;
    let new_token = match new_token {
        Response::Auth(token, _) => token,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
    };
    
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