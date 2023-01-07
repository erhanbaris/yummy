use std::time::Duration;
use general::auth::UserAuth;
use general::config::YummyConfig;
use general::auth::validate_auth;
use general::config::configure_environment;
use general::model::RoomUserType;
use general::state::YummyState;
use general::test::model::AuthenticatedModel;
use general::test::model::RoomCreated;
use general::test::model::UserDisconnectedFromRoom;
use general::test::model::UserJoinedToRoom;

use std::sync::Arc;

use actix::Actor;
use actix::Addr;
use anyhow::Ok;
use database::{create_database, create_connection};

use crate::conn::ConnectionManager;
use crate::room::RoomManager;
use crate::room::model::CreateRoomRequest;
use crate::room::model::JoinToRoomRequest;

use super::AuthManager;
use super::*;

use general::test::DummyClient;

fn create_actor(config: Arc<YummyConfig>) -> anyhow::Result<(Addr<AuthManager<database::SqliteStore>>, Arc<DummyClient>)> {
    let connection = create_connection(":memory:")?;
    #[cfg(feature = "stateless")]
    let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();

    let states = YummyState::new(config.clone(), #[cfg(feature = "stateless")] conn.clone());

    ConnectionManager::new(config.clone(), states.clone(), #[cfg(feature = "stateless")] conn.clone()).start();

    create_database(&mut connection.clone().get()?)?;
    Ok((AuthManager::<database::SqliteStore>::new(config.clone(), states, Arc::new(connection)).start(), Arc::new(DummyClient::default())))
}

/* email unit tests */
#[actix::test]
async fn create_user_via_email() -> anyhow::Result<()> {
    let (address, socket) = create_actor(::general::config::get_configuration())?;
    address.send(EmailAuthRequest {
        user: Arc::new(None),
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
    config.heartbeat_timeout = Duration::from_secs(1);
    
    let (address, socket) = create_actor(Arc::new(config))?;
    address.send(EmailAuthRequest {
        user: Arc::new(None),
        email: "erhanbaris@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: true,
        socket: socket.clone()
    }).await??;

    let auth = socket.clone().auth.lock().unwrap().clone();
    address.send(StartUserTimeout {
        user: Arc::new(Some(UserAuth {
            user: auth.id.deref().clone(),
            session: auth.session.clone()
        })),
        socket: socket.clone()
    }).await??;

    actix::clock::sleep(std::time::Duration::new(3, 0)).await;

    address.send(EmailAuthRequest {
        user: Arc::new(None),
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
        user: Arc::new(None),
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
        user: Arc::new(None),
        email: "erhanbaris@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: true,
        socket: socket.clone()
    }).await??;

    let result = address.send(EmailAuthRequest {
        user: Arc::new(None),
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
    address.send(DeviceIdAuthRequest::new(Arc::new(None), "1234567890".to_string(), socket)).await??;
    Ok(())
}

#[actix::test]
async fn login_user_via_device_id() -> anyhow::Result<()> {
    let mut config = ::general::config::get_configuration().deref().clone();
    config.connection_restore_wait_timeout = Duration::from_secs(1);
    config.heartbeat_interval = Duration::from_secs(1);
    config.heartbeat_timeout = Duration::from_secs(1);
    
    let (address, socket) = create_actor(Arc::new(config))?;
    address.send(DeviceIdAuthRequest::new(Arc::new(None), "1234567890".to_string(), socket.clone())).await??;

    let created_token = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    let auth = socket.clone().auth.lock().unwrap().clone();
    
    address.send(StartUserTimeout {
        user: Arc::new(Some(UserAuth {
            user: auth.id.deref().clone(),
            session: auth.session.clone()
        })),
        socket: socket.clone()
    }).await??;

    actix::clock::sleep(std::time::Duration::new(3, 0)).await;

    address.send(DeviceIdAuthRequest::new(Arc::new(None), "1234567890".to_string(), socket.clone())).await??;
    let logged_in_token = socket.clone().messages.lock().unwrap().pop_back().unwrap();

    assert_ne!(created_token, logged_in_token);

    return Ok(());
}

#[actix::test]
async fn login_users_via_device_id() -> anyhow::Result<()> {
    let (address, socket) = create_actor(::general::config::get_configuration())?;
    address.send(DeviceIdAuthRequest::new(Arc::new(None), "1234567890".to_string(), socket.clone())).await??;
    let login_1 = socket.clone().auth.lock().unwrap().clone();

    address.send(DeviceIdAuthRequest::new(Arc::new(None), "abcdef".to_string(), socket.clone())).await??;
    let login_2 = socket.clone().auth.lock().unwrap().clone();
    assert_ne!(login_1, login_2);

    Ok(())
}

/* custom id unit tests */
#[actix::test]
async fn create_user_via_custom_id() -> anyhow::Result<()> {
    let (address, socket) = create_actor(::general::config::get_configuration())?;
    address.send(CustomIdAuthRequest::new(Arc::new(None), "1234567890".to_string(), socket)).await??;
    Ok(())
}

#[actix::test]
async fn login_user_via_custom_id() -> anyhow::Result<()> {
    let mut config = ::general::config::get_configuration().deref().clone();
    config.connection_restore_wait_timeout = Duration::from_secs(1);
    config.heartbeat_interval = Duration::from_secs(1);
    config.heartbeat_timeout = Duration::from_secs(1);
    
    let (address, socket) = create_actor(Arc::new(config))?;
    address.send(CustomIdAuthRequest::new(Arc::new(None), "1234567890".to_string(), socket.clone())).await??;

    let created_token = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    let auth = socket.clone().auth.lock().unwrap().clone();

    address.send(StartUserTimeout {
        user: Arc::new(Some(UserAuth {
            user: auth.id.deref().clone(),
            session: auth.session.clone()
        })),
        socket: socket.clone()
    }).await??;

    actix::clock::sleep(std::time::Duration::new(3, 0)).await;

    address.send(CustomIdAuthRequest::new(Arc::new(None), "1234567890".to_string(), socket.clone())).await??;
    let logged_in_token = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    assert_ne!(created_token, logged_in_token);

    return Ok(());
}

#[actix::test]
async fn login_users_via_custom_id() -> anyhow::Result<()> {
    let (address, socket) = create_actor(::general::config::get_configuration())?;
    address.send(CustomIdAuthRequest::new(Arc::new(None), "1234567890".to_string(), socket.clone())).await??;
    let login_1 = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    
    address.send(CustomIdAuthRequest::new(Arc::new(None), "abcdef".to_string(), socket.clone())).await??;
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
    address.send(DeviceIdAuthRequest::new(Arc::new(None), "1234567890".to_string(), socket.clone())).await??;
    let old_token: AuthenticatedModel = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let old_token = old_token.token;

    // Wait 1 second
    actix::clock::sleep(std::time::Duration::new(1, 0)).await;
    address.send(RestoreTokenRequest { user: Arc::new(None), token: old_token.to_string(), socket: socket.clone() }).await??;
    let new_token: AuthenticatedModel = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let new_token = new_token.token;
    
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
    address.send(DeviceIdAuthRequest::new(Arc::new(None), "1234567890".to_string(), socket.clone())).await??;
    
    let old_token = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    let old_token: AuthenticatedModel = old_token.into();

    // Wait 3 seconds
    actix::clock::sleep(std::time::Duration::new(3, 0)).await;
    assert!(address.send(RestoreTokenRequest { user: Arc::new(None), token: old_token.token.to_string(), socket: socket.clone() }).await?.is_err());
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
    address.send(DeviceIdAuthRequest::new(Arc::new(None), "1234567890".to_string(), socket.clone())).await??;
    let old_token = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    let old_token: AuthenticatedModel = old_token.into();
    let old_token = old_token.token;

    // Wait 1 second
    actix::clock::sleep(std::time::Duration::new(1, 0)).await;
    address.send(RefreshTokenRequest { user: Arc::new(None), token: old_token.to_string(), socket: socket.clone() }).await??;

    let new_token = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    let new_token: AuthenticatedModel = new_token.into();
    let new_token = new_token.token;

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
        user: Arc::new(None),
        email: "erhanbaris@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: true, socket: socket.clone()
    }).await??;


    let old_token = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    let old_token: AuthenticatedModel = old_token.into();
    let old_token = old_token.token;
    
    // Wait 1 second
    actix::clock::sleep(std::time::Duration::new(1, 0)).await;
    address.send(RefreshTokenRequest{ user: Arc::new(None), token: old_token.clone(), socket: socket.clone() }).await??;
    let new_token = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    let new_token: AuthenticatedModel = new_token.into();
    let new_token = new_token.token;
    
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
async fn logout() -> anyhow::Result<()> {
    let (address, socket) = create_actor(::general::config::get_configuration())?;
    address.send(DeviceIdAuthRequest::new(Arc::new(None), "1234567890".to_string(), socket.clone())).await??;
    let user = socket.clone().auth.lock().unwrap().clone();

    address.send(LogoutRequest {
        user: Arc::new(Some(UserAuth {
            user: user.id.deref().clone(),
            session: user.session
        })),
        socket: socket.clone()
    }).await??;
    Ok(())
}

#[actix::test]
async fn double_login_test() -> anyhow::Result<()> {

    let config = ::general::config::get_configuration();
    let connection = create_connection(":memory:")?;

    #[cfg(feature = "stateless")]
    let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();
    let states = YummyState::new(config.clone(), #[cfg(feature = "stateless")] conn.clone());

    ConnectionManager::new(config.clone(), states.clone(), #[cfg(feature = "stateless")] conn.clone()).start();

    create_database(&mut connection.clone().get()?)?;

    let auth_manager = AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone())).start();
    let room_manager = RoomManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection)).start();

    let user_1_socket = Arc::new(DummyClient::default());
    let user_2_socket = Arc::new(DummyClient::default());

    /* #region Auth */
    auth_manager.send(EmailAuthRequest {
        user: Arc::new(None),
        email: "erhan@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: true,
        socket: user_1_socket.clone()
    }).await??;

    let user_1_auth_jwt = user_1_socket.clone().auth.lock().unwrap().clone();
    let user_1_auth = Arc::new(Some(UserAuth {
        user: user_1_auth_jwt.id.deref().clone(),
        session: user_1_auth_jwt.session.clone()
    }));

    auth_manager.send(EmailAuthRequest {
        user: Arc::new(None),
        email: "baris@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: true,
        socket: user_2_socket.clone()
    }).await??;

    let user_2_auth_jwt = user_2_socket.clone().auth.lock().unwrap().clone();
    let user_2_auth = Arc::new(Some(UserAuth {
        user: user_2_auth_jwt.id.deref().clone(),
        session: user_2_auth_jwt.session.clone()
    }));
    /* #endregion */

    /* #region Room configuration */
    room_manager.send(CreateRoomRequest {
        user: user_1_auth.clone(),
        name: None,
        description: None,
        join_request: false,
        access_type: general::model::CreateRoomAccessType::Public,
        max_user: 4,
        metas: None,
        tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()],
        socket:user_1_socket.clone()
    }).await??;

    let room_id: RoomCreated = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let room_id = room_id.room;

    room_manager.send(JoinToRoomRequest {
        user: user_2_auth.clone(),
        room: room_id,
        room_user_type: RoomUserType::User,
        socket:user_2_socket.clone()
    }).await??;

    let message: UserJoinedToRoom = serde_json::from_str(&user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(&message.class_type[..], "UserJoinedToRoom");
    /* #endregion */

    /* #region Re-auth */
    auth_manager.send(EmailAuthRequest {
        user: user_1_auth.clone(),
        email: "erhan@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: true,
        socket: user_1_socket.clone()
    }).await??;
    /* #endregion */

    /* #region Receive user disconnected message */
    let message: UserDisconnectedFromRoom = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(&message.class_type[..], "UserDisconnectedFromRoom");
    /* #endregion */

    return Ok(());
}


#[actix::test]
async fn user_disconnect_from_room_test() -> anyhow::Result<()> {

    let mut config = ::general::config::get_configuration().deref().clone();
    config.connection_restore_wait_timeout = Duration::from_secs(1);
    config.heartbeat_interval = Duration::from_secs(1);
    config.heartbeat_timeout = Duration::from_secs(1);

    let config = Arc::new(config);

    let connection = create_connection(":memory:")?;

    #[cfg(feature = "stateless")]
    let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();
    let states = YummyState::new(config.clone(), #[cfg(feature = "stateless")] conn.clone());

    ConnectionManager::new(config.clone(), states.clone(), #[cfg(feature = "stateless")] conn.clone()).start();

    create_database(&mut connection.clone().get()?)?;

    let auth_manager = AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone())).start();
    let room_manager = RoomManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection)).start();

    let user_1_socket = Arc::new(DummyClient::default());
    let user_2_socket = Arc::new(DummyClient::default());

    /* #region Auth */
    auth_manager.send(EmailAuthRequest {
        user: Arc::new(None),
        email: "erhan@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: true,
        socket: user_1_socket.clone()
    }).await??;

    let user_1_auth_jwt = user_1_socket.clone().auth.lock().unwrap().clone();
    let user_1_auth = Arc::new(Some(UserAuth {
        user: user_1_auth_jwt.id.deref().clone(),
        session: user_1_auth_jwt.session.clone()
    }));

    auth_manager.send(EmailAuthRequest {
        user: Arc::new(None),
        email: "baris@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: true,
        socket: user_2_socket.clone()
    }).await??;

    let user_2_auth_jwt = user_2_socket.clone().auth.lock().unwrap().clone();
    let user_2_auth = Arc::new(Some(UserAuth {
        user: user_2_auth_jwt.id.deref().clone(),
        session: user_2_auth_jwt.session.clone()
    }));
    /* #endregion */

    /* #region Room configuration */
    room_manager.send(CreateRoomRequest {
        user: user_1_auth.clone(),
        name: None,
        description: None,
        join_request: false,
        access_type: general::model::CreateRoomAccessType::Public,
        max_user: 4,
        metas: None,
        tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()],
        socket:user_1_socket.clone()
    }).await??;

    let room_id: RoomCreated = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let room_id = room_id.room;

    room_manager.send(JoinToRoomRequest {
        user: user_2_auth.clone(),
        room: room_id,
        room_user_type: RoomUserType::User,
        socket:user_2_socket.clone()
    }).await??;

    let message: UserJoinedToRoom = serde_json::from_str(&user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(&message.class_type[..], "UserJoinedToRoom");
    /* #endregion */

    /* #region Start disconnect timeout */
    auth_manager.send(StartUserTimeout {
        user: user_1_auth.clone(),
        socket: user_1_socket.clone()
    }).await??;
    actix::clock::sleep(std::time::Duration::new(3, 0)).await;
    /* #endregion */

    /* #region Receive user disconnected message */
    let message: UserDisconnectedFromRoom = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(&message.class_type[..], "UserDisconnectedFromRoom");
    /* #endregion */

    return Ok(());
}
