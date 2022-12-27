use general::config::configure_environment;
use general::state::RoomUserInformation;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use general::auth::UserAuth;
use general::auth::validate_auth;
use general::config::YummyConfig;
use general::config::get_configuration;
use general::state::YummyState;
use std::env::temp_dir;
use std::sync::Arc;

use actix::Actor;
use actix::Addr;
use anyhow::Ok;
use database::{create_database, create_connection};

use super::*;
use crate::api::auth::AuthManager;
use crate::api::auth::model::*;
use crate::api::conn::ConnectionManager;
use general::web::GenericAnswer;
use general::test::DummyClient;

#[cfg(feature = "stateless")]
use general::test::cleanup_redis;

macro_rules! email_auth {
    ($auth_manager: expr, $config: expr, $email: expr, $password: expr, $create: expr, $recipient: expr) => {
        {
            $auth_manager.send(EmailAuthRequest {
                email: $email,
                password: $password,
                if_not_exist_create: $create,
                socket: $recipient.clone()
            }).await??;
        
            let token: GenericAnswer<String> = $recipient.clone().messages.lock().unwrap().pop_back().unwrap().into();
            let token = token.result.unwrap_or_default();
        
            let user_jwt = validate_auth($config, token).unwrap().user;
            Arc::new(Some(UserAuth {
                user: user_jwt.id,
                session: user_jwt.session
            }))
        }
    };
}

#[derive(Debug, Serialize, Deserialize)]
struct Joined {
    #[serde(rename = "type")]
    class_type: String,
    room_name: Option<String>,
    users: Vec<RoomUserInformation>
}

#[derive(Debug, Serialize, Deserialize)]
struct UserJoinedToRoom {
    #[serde(rename = "type")]
    class_type: String,
    user: UserId,
    room: RoomId,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserDisconnectedFromRoom {
    #[serde(rename = "type")]
    class_type: String,
    user: UserId,
    room: RoomId
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageReceivedFromRoom {
    #[serde(rename = "type")]
    class_type: String,
    user: UserId,
    room: RoomId,
    message: String
}

fn create_actor() -> anyhow::Result<(Addr<RoomManager<database::SqliteStore>>, Addr<AuthManager<database::SqliteStore>>, Arc<YummyConfig>, YummyState, Arc<DummyClient>)> {
    let mut db_location = temp_dir();
    db_location.push(format!("{}.db", Uuid::new_v4()));
    
    configure_environment();

    let mut config = get_configuration().deref().clone();

    #[cfg(feature = "stateless")] {       
        config.redis_prefix = format!("{}:", rand::thread_rng().gen::<usize>().to_string());
    }

    let config = Arc::new(config);
    
    #[cfg(feature = "stateless")]
    let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();

    #[cfg(feature = "stateless")]
    cleanup_redis(conn.clone());
    let states = YummyState::new(config.clone(), #[cfg(feature = "stateless")] conn.clone());

    ConnectionManager::new(config.clone(), states.clone(), #[cfg(feature = "stateless")] conn.clone()).start();

    let connection = create_connection(db_location.to_str().unwrap())?;
    create_database(&mut connection.clone().get()?)?;
    Ok((RoomManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone())).start(), AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection)).start(), config, states.clone(), Arc::new(DummyClient::default())))
}

#[actix::test]
async fn create_room_1() -> anyhow::Result<()> {
    let (room_manager, auth_manager, config, mut states, recipient) = create_actor()?;
    let user = email_auth!(auth_manager, config.clone(), "user@gmail.com".to_string(), "erhan".into(), true, recipient);

    room_manager.send(CreateRoomRequest {
        user: user.clone(),
        disconnect_from_other_room: false,
        name: None,
        access_type: general::model::CreateRoomAccessType::Friend,
        max_user: 4,
        tags: Vec::new(),
        socket:recipient.clone()
    }).await??;

    let room_id: GenericAnswer<RoomId> = recipient.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let room_id = room_id.result.unwrap_or_default();

    let user_id = match user.as_ref() {
        Some(user) => user.user.clone(),
        None => return Err(anyhow::anyhow!("UserId not found"))
    };

    assert!(room_id.get() != uuid::Uuid::nil());
    assert!(states.get_user_room(user_id).is_some());
    
    Ok(())
}

#[actix::test]
async fn create_room_2() -> anyhow::Result<()> {
    let (room_manager, auth_manager, config, mut states, recipient) = create_actor()?;
    let user = email_auth!(auth_manager, config.clone(), "user@gmail.com".to_string(), "erhan".into(), true, recipient);

    room_manager.send(CreateRoomRequest {
        user: user.clone(),
        disconnect_from_other_room: false,
        name: None,
        access_type: general::model::CreateRoomAccessType::Public,
        max_user: 4,
        tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()],
        socket:recipient.clone()
    }).await??;

    let room_id: GenericAnswer<RoomId> = recipient.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let room_id = room_id.result.unwrap_or_default();

    let user_id = match user.as_ref() {
        Some(user) => user.user.clone(),
        None => return Err(anyhow::anyhow!("UserId not found"))
    };

    assert!(room_id.get() != uuid::Uuid::nil());
    assert!(states.get_user_room(user_id).is_some());
    
    Ok(())
}

#[actix::test]
async fn create_room_3() -> anyhow::Result<()> {
    let (room_manager, auth_manager, config, _, user_1_socket) = create_actor()?;
    let user_1 = email_auth!(auth_manager, config.clone(), "user1@gmail.com".to_string(), "erhan".into(), true, user_1_socket);

    let user_2_socket = Arc::new(DummyClient::default());
    let user_2 = email_auth!(auth_manager, config.clone(), "user2@gmail.com".to_string(), "erhan".into(), true, user_2_socket);

    let user_3_socket = Arc::new(DummyClient::default());
    let user_3 = email_auth!(auth_manager, config.clone(), "user3@gmail.com".to_string(), "erhan".into(), true, user_3_socket);


    room_manager.send(CreateRoomRequest {
        user: user_1.clone(),
        disconnect_from_other_room: false,
        name: None,
        access_type: general::model::CreateRoomAccessType::Public,
        max_user: 4,
        tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()],
        socket:user_1_socket.clone()
    }).await??;

    let room_id: GenericAnswer<RoomId> = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let room_id = room_id.result.unwrap_or_default();

    assert!(room_id.get() != uuid::Uuid::nil());

    room_manager.send(JoinToRoomRequest {
        user: user_2.clone(),
        room: room_id,
        room_user_type: RoomUserType::User,
        socket:user_2_socket.clone()
    }).await??;

    let message: GenericAnswer<Joined> = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_front().unwrap()).unwrap();
    let message = message.result.unwrap();
    assert_eq!(&message.class_type[..], "Joined");

    room_manager.send(JoinToRoomRequest {
        user: user_3.clone(),
        room: room_id,
        room_user_type: RoomUserType::User,
        socket:user_3_socket.clone()
    }).await??;

    let message: GenericAnswer<Joined> = serde_json::from_str(&user_3_socket.clone().messages.lock().unwrap().pop_front().unwrap()).unwrap();
    let message = message.result.unwrap();
    assert_eq!(&message.class_type[..], "Joined");

    let user_1_id = user_1.clone().deref().as_ref().unwrap().user.clone();

    // User 1 should receive other 2 users join message
    let message: UserJoinedToRoom = serde_json::from_str(&user_1_socket.clone().messages.lock().unwrap().pop_front().unwrap()).unwrap();
    assert_eq!(message.user, user_2.as_ref().clone().unwrap().user);

    let room_id = message.room;

    let message: UserJoinedToRoom = serde_json::from_str(&user_1_socket.clone().messages.lock().unwrap().pop_front().unwrap()).unwrap();
    assert_eq!(message.user, user_3.as_ref().clone().unwrap().user);
    assert_eq!(&message.class_type[..], "UserJoinedToRoom");

    // User 2 should receive only user 3's join message
    let message: UserJoinedToRoom = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_front().unwrap()).unwrap();
    assert_eq!(message.user, user_3.as_ref().clone().unwrap().user);
    assert_eq!(&message.class_type[..], "UserJoinedToRoom");

    room_manager.send(CreateRoomRequest {
        user: user_1.clone(),
        disconnect_from_other_room: true,
        name: None,
        access_type: general::model::CreateRoomAccessType::Public,
        max_user: 4,
        tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()],
        socket:user_1_socket.clone()
    }).await??;

    let message = user_2_socket.clone().messages.lock().unwrap().pop_front().unwrap();
    let message: UserDisconnectedFromRoom = serde_json::from_str(&message).unwrap();
    assert_eq!(message.user, user_1_id.clone());
    assert_eq!(message.room, room_id.clone());
    assert_eq!(&message.class_type[..], "UserDisconnectedFromRoom");

    let message = user_3_socket.clone().messages.lock().unwrap().pop_front().unwrap();
    let message: UserDisconnectedFromRoom = serde_json::from_str(&message).unwrap();
    assert_eq!(message.user, user_1_id.clone());
    assert_eq!(message.room, room_id.clone());
    assert_eq!(&message.class_type[..], "UserDisconnectedFromRoom");

    Ok(())
}


#[actix::test]
async fn create_room_4() -> anyhow::Result<()> {
    let (room_manager, auth_manager, config, _, user_1_socket) = create_actor()?;
    let user_1 = email_auth!(auth_manager, config.clone(), "user1@gmail.com".to_string(), "erhan".into(), true, user_1_socket);

    let user_2_socket = Arc::new(DummyClient::default());
    let user_2 = email_auth!(auth_manager, config.clone(), "user2@gmail.com".to_string(), "erhan".into(), true, user_2_socket);

    let user_3_socket = Arc::new(DummyClient::default());
    let user_3 = email_auth!(auth_manager, config.clone(), "user3@gmail.com".to_string(), "erhan".into(), true, user_3_socket);

    assert!(room_manager.send(DisconnectFromRoomRequest {
        user: user_1.clone(),
        room: RoomId::default(),
        socket:user_1_socket.clone()
    }).await?.is_err());

    // User 1 should receive other 2 users join message
    let message: Answer = serde_json::from_str(&user_1_socket.clone().messages.lock().unwrap().pop_front().unwrap()).unwrap();
    assert!(!message.status);

    room_manager.send(CreateRoomRequest {
        user: user_1.clone(),
        disconnect_from_other_room: false,
        name: None,
        access_type: general::model::CreateRoomAccessType::Public,
        max_user: 4,
        tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()],
        socket:user_1_socket.clone()
    }).await??;

    let room_id: GenericAnswer<RoomId> = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let room_id = room_id.result.unwrap_or_default();

    assert!(room_id.get() != uuid::Uuid::nil());

    room_manager.send(JoinToRoomRequest {
        user: user_2.clone(),
        room: room_id,
        room_user_type: RoomUserType::User,
        socket:user_2_socket.clone()
    }).await??;

    let message: GenericAnswer<Joined> = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_front().unwrap()).unwrap();
    let message = message.result.unwrap();
    assert_eq!(&message.class_type[..], "Joined");

    room_manager.send(JoinToRoomRequest {
        user: user_3.clone(),
        room: room_id,
        room_user_type: RoomUserType::User,
        socket:user_3_socket.clone()
    }).await??;

    let message: GenericAnswer<Joined> = serde_json::from_str(&user_3_socket.clone().messages.lock().unwrap().pop_front().unwrap()).unwrap();
    let message = message.result.unwrap();
    assert_eq!(&message.class_type[..], "Joined");

    let user_1_id = user_1.clone().deref().as_ref().unwrap().user.clone();

    // User 1 should receive other 2 users join message
    let message: UserJoinedToRoom = serde_json::from_str(&user_1_socket.clone().messages.lock().unwrap().pop_front().unwrap()).unwrap();
    assert_eq!(message.user, user_2.as_ref().clone().unwrap().user);

    let room_id = message.room;

    let message: UserJoinedToRoom = serde_json::from_str(&user_1_socket.clone().messages.lock().unwrap().pop_front().unwrap()).unwrap();
    assert_eq!(message.user, user_3.as_ref().clone().unwrap().user);
    assert_eq!(&message.class_type[..], "UserJoinedToRoom");

    // User 2 should receive only user 3's join message
    let message: UserJoinedToRoom = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_front().unwrap()).unwrap();
    assert_eq!(message.user, user_3.as_ref().clone().unwrap().user);
    assert_eq!(&message.class_type[..], "UserJoinedToRoom");

    assert!(room_manager.send(DisconnectFromRoomRequest {
        user: user_1.clone(),
        room: room_id,
        socket:user_1_socket.clone()
    }).await?.is_ok());

    let message = user_2_socket.clone().messages.lock().unwrap().pop_front().unwrap();
    let message: UserDisconnectedFromRoom = serde_json::from_str(&message).unwrap();
    assert_eq!(message.user, user_1_id.clone());
    assert_eq!(message.room, room_id.clone());
    assert_eq!(&message.class_type[..], "UserDisconnectedFromRoom");

    let message = user_3_socket.clone().messages.lock().unwrap().pop_front().unwrap();
    let message: UserDisconnectedFromRoom = serde_json::from_str(&message).unwrap();
    assert_eq!(message.user, user_1_id.clone());
    assert_eq!(message.room, room_id.clone());
    assert_eq!(&message.class_type[..], "UserDisconnectedFromRoom");

    Ok(())
}

#[actix::test]
async fn message_to_room() -> anyhow::Result<()> {
    let (room_manager, auth_manager, config, _, user_1_socket) = create_actor()?;
    let user_1 = email_auth!(auth_manager, config.clone(), "user1@gmail.com".to_string(), "erhan".into(), true, user_1_socket);
    let user_1_id = user_1.clone().deref().as_ref().unwrap().user.clone();

    let user_2_socket = Arc::new(DummyClient::default());
    let user_2 = email_auth!(auth_manager, config.clone(), "user2@gmail.com".to_string(), "erhan".into(), true, user_2_socket);
    let user_2_id = user_2.clone().deref().as_ref().unwrap().user.clone();

    let user_3_socket = Arc::new(DummyClient::default());
    let user_3 = email_auth!(auth_manager, config.clone(), "user3@gmail.com".to_string(), "erhan".into(), true, user_3_socket);

    room_manager.send(CreateRoomRequest {
        user: user_1.clone(),
        disconnect_from_other_room: false,
        name: None,
        access_type: general::model::CreateRoomAccessType::Public,
        max_user: 4,
        tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()],
        socket:user_1_socket.clone()
    }).await??;

    let room_id: GenericAnswer<RoomId> = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let room_id = room_id.result.unwrap_or_default();

    assert!(room_id.get() != uuid::Uuid::nil());

    // Join to room
    room_manager.send(JoinToRoomRequest {
        user: user_2.clone(),
        room: room_id,
        room_user_type: RoomUserType::User,
        socket:user_2_socket.clone()
    }).await??;

    room_manager.send(JoinToRoomRequest {
        user: user_3.clone(),
        room: room_id,
        room_user_type: RoomUserType::User,
        socket:user_3_socket.clone()
    }).await??;

    // Send message to room
    room_manager.send(MessageToRoomRequest {
        user: user_1.clone(),
        room: room_id,
        message: "HELLO".to_string(),
        socket:user_1_socket.clone()
    }).await??;

    // All users will receive the message
    let message = user_2_socket.clone().messages.lock().unwrap().pop_back().unwrap();    
    let message = serde_json::from_str::<MessageReceivedFromRoom>(&message).unwrap();
    assert_eq!(message.user, user_1_id.clone());
    assert_eq!(message.room, room_id.clone());
    assert_eq!(&message.message, "HELLO");
    assert_eq!(&message.class_type[..], "MessageFromRoom");

    let message = serde_json::from_str::<MessageReceivedFromRoom>(&user_3_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(message.user, user_1_id.clone());
    assert_eq!(message.room, room_id.clone());
    assert_eq!(&message.message, "HELLO");
    assert_eq!(&message.class_type[..], "MessageFromRoom");


    // Send message to room
    room_manager.send(MessageToRoomRequest {
        user: user_2.clone(),
        room: room_id,
        message: "WORLD".to_string(),
        socket:user_2_socket.clone()
    }).await??;

    // All users will receive the message
    let message = serde_json::from_str::<MessageReceivedFromRoom>(&user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(message.user, user_2_id.clone());
    assert_eq!(message.room, room_id.clone());
    assert_eq!(&message.message, "WORLD");
    assert_eq!(&message.class_type[..], "MessageFromRoom");

    let message = serde_json::from_str::<MessageReceivedFromRoom>(&user_3_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(message.user, user_2_id.clone());
    assert_eq!(message.room, room_id.clone());
    assert_eq!(&message.message, "WORLD");
    assert_eq!(&message.class_type[..], "MessageFromRoom");

    Ok(())
}

#[actix::test]
async fn get_rooms() -> anyhow::Result<()> {
    let (room_manager, auth_manager, config, _, user_1_socket) = create_actor()?;
    email_auth!(auth_manager, config.clone(), "user@gmail.com".to_string(), "erhan".into(), true, user_1_socket);

    for i in 0..100 {
        let user_socket = Arc::new(DummyClient::default());
        let user = email_auth!(auth_manager, config.clone(), format!("user{}@gmail.com", i), "erhan".into(), true, user_socket.clone());

        room_manager.send(CreateRoomRequest {
            user,
            disconnect_from_other_room: false,
            name: None,
            access_type: general::model::CreateRoomAccessType::Public,
            max_user: 4,
            tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()],
            socket:user_socket.clone()
        }).await??;
    }

    room_manager.send(RoomListRequet {
        socket: user_1_socket.clone(),
        members: Vec::new(),
        tag: None
    }).await??;

    let result: GenericAnswer<serde_json::Value> = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();

    assert!(result.status);
    assert!(result.result.is_some());
    let items = result.result.unwrap();
    if let serde_json::Value::Array(items) = items {
        assert_eq!(items.len(), 100);
        let first_item = items.get(0).unwrap();

        if let serde_json::Value::Object(obj) = first_item {
            assert!(obj.contains_key("id"));
            assert!(obj.contains_key("name"));
            assert!(obj.contains_key("user-length"));
            assert!(obj.contains_key("max-user"));
            assert!(obj.contains_key("users"));
            assert!(obj.contains_key("tags"));
            assert!(obj.contains_key("access-type"));
            assert!(obj.contains_key("insert-date"));
        } else { 
            assert!(false, "Item is not object");
        }
    } else { 
        assert!(false, "Return valus is not array");
    }

    Ok(())
}
