#[allow(unused_mut)]

use general::config::configure_environment;
use general::model::CreateRoomAccessType;
use general::state::RoomUserInformation;
use general::test::model::*;
use uuid::Uuid;

use general::auth::UserAuth;
use general::auth::validate_auth;
use general::config::YummyConfig;
use general::config::get_configuration;
use general::state::YummyState;
use general::web::GenericAnswer;
use general::test::DummyClient;

use std::env::temp_dir;
use std::sync::Arc;

use actix::Actor;
use actix::Addr;
use anyhow::Ok;
use database::{create_database, create_connection};

use super::*;
use crate::auth::AuthManager;
use crate::auth::model::*;
use crate::conn::ConnectionManager;
use crate::plugin::PluginExecuter;


macro_rules! email_auth {
    ($auth_manager: expr, $config: expr, $email: expr, $password: expr, $create: expr, $recipient: expr) => {
        {
            $auth_manager.send(EmailAuthRequest {
                request_id: None,
                auth: Arc::new(None),
                email: $email,
                password: $password,
                if_not_exist_create: $create,
                socket: $recipient.clone()
            }).await??;
        
            let token: AuthenticatedModel = $recipient.clone().messages.lock().unwrap().pop_back().unwrap().into();
            let token = token.token;
        
            let user_jwt = validate_auth($config, token).unwrap().user;
            Arc::new(Some(UserAuth {
                user: user_jwt.id.deref().clone(),
                session: user_jwt.session
            }))
        }
    };
}

fn create_actor() -> anyhow::Result<(Addr<RoomManager<database::SqliteStore>>, Addr<AuthManager<database::SqliteStore>>, Arc<YummyConfig>, YummyState, Arc<DummyClient>)> {
    let mut db_location = temp_dir();
    db_location.push(format!("{}.db", Uuid::new_v4()));
    
    configure_environment();

    let mut config = get_configuration().deref().clone();

    #[cfg(feature = "stateless")] {
        use rand::Rng;     
        config.redis_prefix = format!("{}:", rand::thread_rng().gen::<usize>().to_string());
    }

    let config = Arc::new(config);
    
    #[cfg(feature = "stateless")]
    let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();

    let states = YummyState::new(config.clone(), #[cfg(feature = "stateless")] conn.clone());
    let executer = Arc::new(PluginExecuter::default());

    ConnectionManager::new(config.clone(), states.clone(), executer.clone(), #[cfg(feature = "stateless")] conn.clone()).start();

    let connection = create_connection(db_location.to_str().unwrap())?;
    create_database(&mut connection.clone().get()?)?;
    Ok((RoomManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone()), executer.clone()).start(), AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection), executer).start(), config, states.clone(), Arc::new(DummyClient::default())))
}

#[actix::test]
async fn create_room_1() -> anyhow::Result<()> {
    let (room_manager, auth_manager, config, mut states, recipient) = create_actor()?;
    let user = email_auth!(auth_manager, config.clone(), "user@gmail.com".to_string(), "erhan".into(), true, recipient);

    room_manager.send(CreateRoomRequest {
        request_id: None,
        auth: user.clone(),
        name: None,
        description: None,
        join_request: false,
        access_type: general::model::CreateRoomAccessType::Friend,
        max_user: 4,
        metas: None,
        tags: Vec::new(),
        socket:recipient.clone()
    }).await??;

    let room_id: RoomCreated = recipient.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let room_id = room_id.room;

    let session_id = match user.as_ref() {
        Some(user) => &user.session,
        None => return Err(anyhow::anyhow!("UserId not found"))
    };

    assert!(!room_id.is_empty());
    assert!(states.get_user_rooms(session_id).unwrap().len() == 1);
    
    Ok(())
}

#[actix::test]
async fn create_room_2() -> anyhow::Result<()> {
    let (room_manager, auth_manager, config, mut states, recipient) = create_actor()?;
    let user = email_auth!(auth_manager, config.clone(), "user@gmail.com".to_string(), "erhan".into(), true, recipient);

    room_manager.send(CreateRoomRequest {
        request_id: None,
        auth: user.clone(),
        name: None,
        description: None,
        join_request: false,
        access_type: general::model::CreateRoomAccessType::Public,
        max_user: 4,
        metas: None,
        tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()],
        socket:recipient.clone()
    }).await??;

    let room_created: RoomCreated = recipient.clone().messages.lock().unwrap().pop_back().unwrap().into();

    let session_id = match user.as_ref() {
        Some(user) => &user.session,
        None => return Err(anyhow::anyhow!("UserId not found"))
    };

    assert!(!room_created.room.is_empty());
    assert!(states.get_user_rooms(session_id).unwrap().len() == 1);
    
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
        request_id: None,
        auth: user_1.clone(),
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

    assert!(!room_id.is_empty());

    room_manager.send(JoinToRoomRequest {
        request_id: None,
        auth: user_2.clone(),
        room: room_id,
        room_user_type: RoomUserType::User,
        socket:user_2_socket.clone()
    }).await??;

    let message: GenericAnswer<general::test::model::Joined> = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_front().unwrap()).unwrap();
    let message = message.result;
    assert_eq!(&message.class_type[..], "Joined");

    room_manager.send(JoinToRoomRequest {
        request_id: None,
        auth: user_3.clone(),
        room: room_id,
        room_user_type: RoomUserType::User,
        socket:user_3_socket.clone()
    }).await??;

    let message: GenericAnswer<general::test::model::Joined> = serde_json::from_str(&user_3_socket.clone().messages.lock().unwrap().pop_front().unwrap()).unwrap();
    let message = message.result;
    assert_eq!(&message.class_type[..], "Joined");

    // User 1 should receive other 2 users join message
    let message: UserJoinedToRoom = serde_json::from_str(&user_1_socket.clone().messages.lock().unwrap().pop_front().unwrap()).unwrap();
    assert_eq!(message.user, user_2.as_ref().clone().unwrap().user);

    let message: UserJoinedToRoom = serde_json::from_str(&user_1_socket.clone().messages.lock().unwrap().pop_front().unwrap()).unwrap();
    assert_eq!(message.user, user_3.as_ref().clone().unwrap().user);
    assert_eq!(&message.class_type[..], "UserJoinedToRoom");

    // User 2 should receive only user 3's join message
    let message: UserJoinedToRoom = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_front().unwrap()).unwrap();
    assert_eq!(message.user, user_3.as_ref().clone().unwrap().user);
    assert_eq!(&message.class_type[..], "UserJoinedToRoom");

    room_manager.send(CreateRoomRequest {
        request_id: None,
        auth: user_1.clone(),
        name: None,
        description: None,
        join_request: false,
        access_type: general::model::CreateRoomAccessType::Public,
        max_user: 4,
        metas: None,
        tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()],
        socket:user_1_socket.clone()
    }).await??;

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

    room_manager.send(CreateRoomRequest {
        request_id: None,
        auth: user_1.clone(),
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

    assert!(!room_id.is_empty());

    room_manager.send(JoinToRoomRequest {
        request_id: None,
        auth: user_2.clone(),
        room: room_id,
        room_user_type: RoomUserType::User,
        socket:user_2_socket.clone()
    }).await??;

    let message: GenericAnswer<general::test::model::Joined> = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_front().unwrap()).unwrap();
    let message = message.result;
    assert_eq!(&message.class_type[..], "Joined");

    room_manager.send(JoinToRoomRequest {
        request_id: None,
        auth: user_3.clone(),
        room: room_id,
        room_user_type: RoomUserType::User,
        socket:user_3_socket.clone()
    }).await??;

    let message: GenericAnswer<general::test::model::Joined> = serde_json::from_str(&user_3_socket.clone().messages.lock().unwrap().pop_front().unwrap()).unwrap();
    let message = message.result;
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

    room_manager.send(DisconnectFromRoomRequest {
        request_id: None,
        auth: user_1.clone(),
        room: room_id,
        socket:user_1_socket.clone()
    }).await.unwrap();

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
        request_id: None,
        auth: user_1.clone(),
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

    assert!(!room_id.is_empty());

    // Join to room
    room_manager.send(JoinToRoomRequest {
        request_id: None,
        auth: user_2.clone(),
        room: room_id,
        room_user_type: RoomUserType::User,
        socket:user_2_socket.clone()
    }).await??;

    room_manager.send(JoinToRoomRequest {
        request_id: None,
        auth: user_3.clone(),
        room: room_id,
        room_user_type: RoomUserType::User,
        socket:user_3_socket.clone()
    }).await??;

    // Send message to room
    room_manager.send(MessageToRoomRequest {
        request_id: None,
        auth: user_1.clone(),
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
        request_id: None,
        auth: user_2.clone(),
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
        let auth = email_auth!(auth_manager, config.clone(), format!("user{}@gmail.com", i), "erhan".into(), true, user_socket.clone());

        room_manager.send(CreateRoomRequest {
            request_id: None,
            auth,
            name: None,
            description: None,
            join_request: false,
            access_type: general::model::CreateRoomAccessType::Public,
            max_user: 4,
            metas: None,
            tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()],
            socket:user_socket.clone()
        }).await??;
    }

    room_manager.send(RoomListRequest {
        request_id: None,
        socket: user_1_socket.clone(),
        members: Vec::new(),
        tag: None
    }).await??;

    let result: GenericAnswer<serde_json::Value> = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();

    assert!(result.status);
    let items = result.result;
    if let Some(serde_json::Value::Array(items)) = items.get("rooms") {
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
        assert!(false, "Return value is not array");
    }

    Ok(())
}

#[actix::test]
async fn room_meta_check() -> anyhow::Result<()> {
    let (room_manager, auth_manager, config, _, user_1_socket) = create_actor()?;
    let user_1 = email_auth!(auth_manager, config.clone(), "user1@gmail.com".to_string(), "erhan".into(), true, user_1_socket);
    let user_1_id = user_1.clone().deref().as_ref().unwrap().user.clone();

    let user_2_socket = Arc::new(DummyClient::default());
    let user_2 = email_auth!(auth_manager, config.clone(), "user2@gmail.com".to_string(), "erhan".into(), true, user_2_socket);
    let user_2_id = user_2.clone().deref().as_ref().unwrap().user.clone();

    room_manager.send(CreateRoomRequest {
        request_id: None,
        auth: user_1.clone(),
        name: None,
        description: None,
        join_request: false,
        access_type: general::model::CreateRoomAccessType::Public,
        max_user: 4,
        metas: Some(HashMap::from([
            ("gender".to_string(), MetaType::String("Male".to_string(), RoomMetaAccess::User)),
            ("location".to_string(), MetaType::String("Copenhagen".to_string(), RoomMetaAccess::User)),
            ("score".to_string(), MetaType::Number(15.3, RoomMetaAccess::Anonymous)),
        ])),
        tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()],
        socket:user_1_socket.clone()
    }).await??;

    let room_id: RoomCreated = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let room_id = room_id.room;

    room_manager.send(JoinToRoomRequest {
        request_id: None,
        auth: user_2.clone(),
        room: room_id,
        room_user_type: RoomUserType::User,
        socket:user_2_socket.clone()
    }).await??;

    // Get room information
    room_manager.send(GetRoomRequest {
        request_id: None,
        auth: user_1,
        socket: user_1_socket.clone(),
        members: Vec::new(),
        room: room_id.clone()
    }).await??;

    let room_info: GenericAnswer<serde_json::Value> = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    assert!(room_info.status);

    let room_info = room_info.result;

    let access_type: CreateRoomAccessType = serde_json::from_value(room_info.get("access-type").unwrap().clone())?;
    let max_user: i32 = serde_json::from_value(room_info.get("max-user").unwrap().clone())?;
    let metas: HashMap<String, serde_json::Value> = serde_json::from_value(room_info.get("metas").unwrap().clone())?;
    let mut tags: Vec<String> = serde_json::from_value(room_info.get("tags").unwrap().clone())?;
    let mut users: Vec<RoomUserInformation> = serde_json::from_value(room_info.get("users").unwrap().clone())?;

    assert_eq!(serde_json::from_value::<String>(metas.get("gender").unwrap().clone()).unwrap(), "Male".to_string());
    assert_eq!(serde_json::from_value::<String>(metas.get("location").unwrap().clone()).unwrap(), "Copenhagen".to_string());
    assert_eq!(serde_json::from_value::<f64>(metas.get("score").unwrap().clone()).unwrap(), 15.3);


    assert_eq!(access_type, CreateRoomAccessType::Public);
    assert_eq!(max_user, 4);
    tags.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(tags, vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()]);
    assert_eq!(users.len(), 2);
    
    users.sort_by(|a, b| a.user_type.partial_cmp(&b.user_type).unwrap());
    assert_eq!(users[1].user_type, RoomUserType::Owner);
    assert_eq!(users[1].name, None);
    assert_eq!(users[1].user_id.deref(), &user_1_id);

    assert_eq!(users[0].user_type, RoomUserType::User);
    assert_eq!(users[0].name, None);
    assert_eq!(users[0].user_id.deref(), &user_2_id);

    Ok(())
}


#[actix::test]
async fn room_meta_update() -> anyhow::Result<()> {
    let (room_manager, auth_manager, config, _, user_1_socket) = create_actor()?;
    let user_1 = email_auth!(auth_manager, config.clone(), "user1@gmail.com".to_string(), "erhan".into(), true, user_1_socket);
    let user_1_id = user_1.clone().deref().as_ref().unwrap().user.clone();

    room_manager.send(CreateRoomRequest {
        request_id: None,
        auth: user_1.clone(),
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

    room_manager.send(UpdateRoom {
        request_id: None,
        auth: user_1.clone(),
        room_id,
        name: Some("Erhan".to_string()),
        description: None,
        join_request: None,
        meta_action: Some(MetaAction::OnlyAddOrUpdate),
        access_type: None,
        max_user: None,
        tags: None,
        user_permission: None,
        socket: user_1_socket.clone(),
        metas: Some(HashMap::from([
            ("1".to_string(), MetaType::Bool(true, RoomMetaAccess::Anonymous)),
            ("2".to_string(), MetaType::Bool(true, RoomMetaAccess::Anonymous)),
            ("3".to_string(), MetaType::Bool(true, RoomMetaAccess::Anonymous)),
            ("4".to_string(), MetaType::Bool(true, RoomMetaAccess::Anonymous)),
            ("5".to_string(), MetaType::Bool(true, RoomMetaAccess::Anonymous)),
            ("6".to_string(), MetaType::Bool(true, RoomMetaAccess::Anonymous)),
            ("7".to_string(), MetaType::Bool(true, RoomMetaAccess::Anonymous)),
            ("8".to_string(), MetaType::Bool(true, RoomMetaAccess::Anonymous)),
            ("9".to_string(), MetaType::Bool(true, RoomMetaAccess::Anonymous)),
        ]))
    }).await??;

    /* Check room metas */
    room_manager.send(GetRoomRequest {
        request_id: None,
        auth: user_1.clone(),
        socket: user_1_socket.clone(),
        members: Vec::new(),
        room: room_id.clone()
    }).await??;

    let room_info: GenericAnswer<serde_json::Value> = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    assert!(room_info.status);

    let room_info = room_info.result;
    let metas: HashMap<String, serde_json::Value> = serde_json::from_value(room_info.get("metas").unwrap().clone())?;
    assert_eq!(metas.len(), 9);
    

    /* Add new meta and keep old metas */
    room_manager.send(UpdateRoom {
        request_id: None,
        auth: user_1.clone(),
        room_id,
        name: Some("Erhan".to_string()),
        description: None,
        join_request: None,
        meta_action: Some(MetaAction::OnlyAddOrUpdate),
        access_type: None,
        max_user: None,
        tags: None,
        user_permission: None,
        socket: user_1_socket.clone(),
        metas: Some(HashMap::from([
            ("10".to_string(), MetaType::Bool(true, RoomMetaAccess::Anonymous)),
        ]))
    }).await??;

    /* Check room metas */
    room_manager.send(GetRoomRequest {
        request_id: None,
        auth: user_1.clone(),
        socket: user_1_socket.clone(),
        members: Vec::new(),
        room: room_id.clone()
    }).await??;

    let room_info: GenericAnswer<serde_json::Value> = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    assert!(room_info.status);

    let room_info = room_info.result;
    let metas: HashMap<String, serde_json::Value> = serde_json::from_value(room_info.get("metas").unwrap().clone())?;
    assert_eq!(metas.len(), 10);
    

    /* Add and update new meta but remove unused metas */
    room_manager.send(UpdateRoom {
        request_id: None,
        auth: user_1.clone(),
        room_id,
        name: Some("Erhan".to_string()),
        description: None,
        join_request: None,
        meta_action: Some(MetaAction::RemoveUnusedMetas),
        access_type: None,
        max_user: None,
        tags: None,
        user_permission: None,
        socket: user_1_socket.clone(),
        metas: Some(HashMap::from([
            ("10".to_string(), MetaType::Bool(true, RoomMetaAccess::User)),
            ("11".to_string(), MetaType::Bool(true, RoomMetaAccess::Anonymous)),
        ]))
    }).await??;

    /* Check room metas */
    room_manager.send(GetRoomRequest {
        request_id: None,
        auth: user_1.clone(),
        socket: user_1_socket.clone(),
        members: Vec::new(),
        room: room_id.clone()
    }).await??;

    let room_info: GenericAnswer<serde_json::Value> = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    assert!(room_info.status);

    let room_info = room_info.result;
    let metas: HashMap<String, serde_json::Value> = serde_json::from_value(room_info.get("metas").unwrap().clone())?;
    assert_eq!(metas.len(), 2);
    

    /* Remove all metas */
    room_manager.send(UpdateRoom {
        request_id: None,
        auth: user_1.clone(),
        room_id,
        name: Some("Erhan".to_string()),
        description: Some("Erhan".to_string()),
        join_request: Some(true),
        meta_action: Some(MetaAction::RemoveAllMetas),
        access_type: Some(CreateRoomAccessType::Private),
        max_user: Some(512),
        tags: None,
        user_permission: Some(HashMap::from([(user_1_id, RoomUserType::User)])),
        socket: user_1_socket.clone(),
        metas: Some(HashMap::from([
            ("12".to_string(), MetaType::Bool(true, RoomMetaAccess::Anonymous)),
        ]))
    }).await??;

    /* Check room metas */
    room_manager.send(GetRoomRequest {
        request_id: None,
        auth: user_1.clone(),
        socket: user_1_socket.clone(),
        members: Vec::new(),
        room: room_id.clone()
    }).await??;

    let room_info: GenericAnswer<serde_json::Value> = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    assert!(room_info.status);

    let room_info = room_info.result;
    let metas: HashMap<String, serde_json::Value> = serde_json::from_value(room_info.get("metas").unwrap().clone())?;
    
    assert_eq!(metas.len(), 0);
    

    Ok(())
}

#[actix::test]
async fn room_update() -> anyhow::Result<()> {
    let (room_manager, auth_manager, config, _, user_1_socket) = create_actor()?;
    let user_1 = email_auth!(auth_manager, config.clone(), "user1@gmail.com".to_string(), "erhan".into(), true, user_1_socket);

    let user_2_socket = Arc::new(DummyClient::default());
    let user_2 = email_auth!(auth_manager, config.clone(), "user2@gmail.com".to_string(), "erhan".into(), true, user_2_socket);

    room_manager.send(CreateRoomRequest {
        request_id: None,
        auth: user_1.clone(),
        name: None,
        description: None,
        join_request: false,
        access_type: general::model::CreateRoomAccessType::Public,
        max_user: 4,
        metas: Some(HashMap::from([
            ("gender".to_string(), MetaType::String("Male".to_string(), RoomMetaAccess::User)),
            ("location".to_string(), MetaType::String("Copenhagen".to_string(), RoomMetaAccess::User)),
            ("score".to_string(), MetaType::Number(15.3, RoomMetaAccess::Anonymous)),
            ("other".to_string(), MetaType::Bool(true, RoomMetaAccess::Anonymous)),
        ])),
        tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()],
        socket:user_1_socket.clone()
    }).await??;

    let room_id: RoomCreated = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let room_id = room_id.room;

    room_manager.send(JoinToRoomRequest {
        request_id: None,
        auth: user_2.clone(),
        room: room_id,
        room_user_type: RoomUserType::User,
        socket:user_2_socket.clone()
    }).await??;

    // Get room information
    room_manager.send(GetRoomRequest {
        request_id: None,
        auth: user_1.clone(),
        socket: user_1_socket.clone(),
        members: Vec::new(),
        room: room_id.clone()
    }).await??;

    let room_info: GenericAnswer<serde_json::Value> = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    assert!(room_info.status);

    let room_info = room_info.result;
    let metas: HashMap<String, serde_json::Value> = serde_json::from_value(room_info.get("metas").unwrap().clone())?;

    assert_eq!(serde_json::from_value::<String>(metas.get("gender").unwrap().clone()).unwrap(), "Male".to_string());
    assert_eq!(serde_json::from_value::<String>(metas.get("location").unwrap().clone()).unwrap(), "Copenhagen".to_string());
    assert_eq!(serde_json::from_value::<f64>(metas.get("score").unwrap().clone()).unwrap(), 15.3);
    
    room_manager.send(UpdateRoom {
        request_id: None,
        auth: user_1.clone(),
        name: None,
        description: None,
        room_id: room_id.clone(),
        meta_action: None,
        join_request: None,
        access_type: None,
        user_permission: None,
        max_user: None,
        metas: Some(HashMap::from([
            ("gender".to_string(), MetaType::String("Female".to_string(), RoomMetaAccess::User)),
            ("location".to_string(), MetaType::String("oslo".to_string(), RoomMetaAccess::User)),
            ("score".to_string(), MetaType::Number(30.0, RoomMetaAccess::Anonymous)),
        ])),
        tags: Some(vec!["new tag".to_string()]),
        socket:user_1_socket.clone(),
    }).await??;

    // Get room information
    room_manager.send(GetRoomRequest {
        request_id: None,
        auth: user_1.clone(),
        socket: user_1_socket.clone(),
        members: Vec::new(),
        room: room_id.clone()
    }).await??;

    let room_info: GenericAnswer<serde_json::Value> = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    assert!(room_info.status);

    let room_info = room_info.result;
    let metas: HashMap<String, serde_json::Value> = serde_json::from_value(room_info.get("metas").unwrap().clone())?;

    assert_eq!(serde_json::from_value::<String>(metas.get("gender").unwrap().clone()).unwrap(), "Female".to_string());
    assert_eq!(serde_json::from_value::<String>(metas.get("location").unwrap().clone()).unwrap(), "oslo".to_string());
    assert_eq!(serde_json::from_value::<f64>(metas.get("score").unwrap().clone()).unwrap(), 30.0);


    Ok(())
}

#[allow(unused_macros)]
macro_rules! message_received_from_room {
    ($socket: expr, $sender: expr, $room: expr, $message: expr) => {
        let message = $socket.clone().messages.lock().unwrap().pop_back().unwrap();    
        let message = serde_json::from_str::<MessageReceivedFromRoom>(&message).unwrap();
        assert_eq!(message.user, $sender.clone());
        assert_eq!(message.room, $room.clone());
        assert_eq!(&message.message, $message);
        assert_eq!(&message.class_type[..], "MessageFromRoom");
        }
}

#[actix::test]
async fn multi_room_support() -> anyhow::Result<()> {

    let config = ::general::config::get_configuration();
    let connection = create_connection(":memory:")?;

    #[cfg(feature = "stateless")]
    let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();
    let states = YummyState::new(config.clone(), #[cfg(feature = "stateless")] conn.clone());
    let executer = Arc::new(PluginExecuter::default());

    ConnectionManager::new(config.clone(), states.clone(), executer.clone(), #[cfg(feature = "stateless")] conn.clone()).start();

    create_database(&mut connection.clone().get()?)?;

    let auth_manager = AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone()), executer.clone()).start();
    let room_manager = RoomManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection), executer.clone()).start();

    let user_1_socket = Arc::new(DummyClient::default());
    let user_2_socket = Arc::new(DummyClient::default());

    /* #region Auth */
    auth_manager.send(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
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
        request_id: None,
        auth: Arc::new(None),
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
        request_id: None,
        auth: user_1_auth.clone(),
        name: None,
        description: None,
        join_request: false,
        access_type: general::model::CreateRoomAccessType::Public,
        max_user: 4,
        metas: None,
        tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()],
        socket:user_1_socket.clone()
    }).await??;

    let room_1_id: RoomCreated = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let room_1_id = room_1_id.room;

    room_manager.send(CreateRoomRequest {
        request_id: None,
        auth: user_1_auth.clone(),
        name: None,
        description: None,
        join_request: false,
        access_type: general::model::CreateRoomAccessType::Public,
        max_user: 4,
        metas: None,
        tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()],
        socket:user_1_socket.clone()
    }).await??;

    let room_2_id: RoomCreated = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let room_2_id = room_2_id.room;

    room_manager.send(JoinToRoomRequest {
        request_id: None,
        auth: user_2_auth.clone(),
        room: room_1_id,
        room_user_type: RoomUserType::User,
        socket:user_2_socket.clone()
    }).await??;

    let message: UserJoinedToRoom = serde_json::from_str(&user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(&message.class_type[..], "UserJoinedToRoom");


    room_manager.send(JoinToRoomRequest {
        request_id: None,
        auth: user_2_auth.clone(),
        room: room_2_id,
        room_user_type: RoomUserType::User,
        socket:user_2_socket.clone()
    }).await??;

    let message: UserJoinedToRoom = serde_json::from_str(&user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(&message.class_type[..], "UserJoinedToRoom");
    assert_eq!(&message.room, &room_2_id);
    /* #endregion */

    /* #region Send messages to room */
    room_manager.send(MessageToRoomRequest {
        request_id: None,
        auth: user_1_auth.clone(),
        room: room_1_id,
        message: "hello 1".to_string(),
        socket:user_1_socket.clone()
    }).await??;

    message_received_from_room!(user_2_socket, user_1_auth_jwt.id.deref(), room_1_id, "hello 1");

    room_manager.send(MessageToRoomRequest {
        request_id: None,
        auth: user_1_auth.clone(),
        room: room_2_id,
        message: "hello 2".to_string(),
        socket:user_1_socket.clone()
    }).await??;

    message_received_from_room!(user_2_socket, user_1_auth_jwt.id.deref(), room_2_id, "hello 2");

    room_manager.send(MessageToRoomRequest {
        request_id: None,
        auth: user_2_auth.clone(),
        room: room_1_id,
        message: "world 1".to_string(),
        socket:user_2_socket.clone()
    }).await??;

    message_received_from_room!(user_1_socket, user_2_auth_jwt.id.deref(), room_1_id, "world 1");

    room_manager.send(MessageToRoomRequest {
        request_id: None,
        auth: user_2_auth.clone(),
        room: room_2_id,
        message: "world 2".to_string(),
        socket:user_2_socket.clone()
    }).await??;

    message_received_from_room!(user_1_socket, user_2_auth_jwt.id.deref(), room_2_id, "world 2");
    /* #endregion */

    return Ok(());
}

#[actix::test]
async fn room_join_request_approve() -> anyhow::Result<()> {

    let config = ::general::config::get_configuration();
    let connection = create_connection(":memory:")?;

    #[cfg(feature = "stateless")]
    let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();
    let states = YummyState::new(config.clone(), #[cfg(feature = "stateless")] conn.clone());
    let executer = Arc::new(PluginExecuter::default());

    ConnectionManager::new(config.clone(), states.clone(), executer.clone(), #[cfg(feature = "stateless")] conn.clone()).start();

    create_database(&mut connection.clone().get()?)?;

    let auth_manager = AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone()), executer.clone()).start();
    let room_manager = RoomManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection), executer.clone()).start();

    let user_1_socket = Arc::new(DummyClient::default());
    let user_2_socket = Arc::new(DummyClient::default());

    /* #region Auth */
    auth_manager.send(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
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
        request_id: None,
        auth: Arc::new(None),
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
        request_id: None,
        auth: user_1_auth.clone(),
        name: None,
        description: None,
        join_request: true,
        access_type: general::model::CreateRoomAccessType::Public,
        max_user: 4,
        metas: None,
        tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()],
        socket:user_1_socket.clone()
    }).await??;
    
    let room_1_id: RoomCreated = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let room_1_id = room_1_id.room;

    room_manager.send(JoinToRoomRequest {
        request_id: None,
        auth: user_2_auth.clone(),
        room: room_1_id,
        room_user_type: RoomUserType::User,
        socket:user_2_socket.clone()
    }).await??;

    let message: NewRoomJoinRequest = serde_json::from_str(&user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(&message.class_type[..], "NewJoinRequest");
    assert_eq!(&message.room, &room_1_id);
    assert_eq!(&message.user, user_2_auth_jwt.id.deref());
    assert_eq!(message.user_type, RoomUserType::User);

    let message: JoinRequested = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(&message.class_type[..], "JoinRequested");
    assert_eq!(&message.room, &room_1_id);

    // Get waiting list
    room_manager.send(WaitingRoomJoins {
        request_id: None,
        auth: user_1_auth.clone(),
        room: room_1_id,
        socket:user_1_socket.clone()
    }).await??;

    let message: WaitingRoomJoinsResponse = serde_json::from_str(&user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(&message.class_type[..], "WaitingRoomJoins");
    assert_eq!(&message.room, &room_1_id);
    assert_eq!(&message.users, &HashMap::from([(user_2_auth_jwt.id.deref().clone(), RoomUserType::User)]));

    // Approve waiting user
    room_manager.send(ProcessWaitingUser {
        request_id: None,
        auth: user_1_auth.clone(),
        room: room_1_id,
        user: user_2_auth_jwt.id.deref().clone(),
        status: true,
        socket: user_1_socket.clone()
    }).await??;

    let message: GenericAnswer<general::test::model::Joined> = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    let message = message.result;
    assert_eq!(&message.class_type[..], "Joined");

    let message: Answer = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert!(message.status);
    
    /* #endregion */

    return Ok(());
}


#[actix::test]
async fn room_join_request_decline() -> anyhow::Result<()> {

    let config = ::general::config::get_configuration();
    let connection = create_connection(":memory:")?;

    #[cfg(feature = "stateless")]
    let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();
    let states = YummyState::new(config.clone(), #[cfg(feature = "stateless")] conn.clone());
    let executer = Arc::new(PluginExecuter::default());

    ConnectionManager::new(config.clone(), states.clone(), executer.clone(), #[cfg(feature = "stateless")] conn.clone()).start();

    create_database(&mut connection.clone().get()?)?;

    let auth_manager = AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone()), executer.clone()).start();
    let room_manager = RoomManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection), executer.clone()).start();

    let user_1_socket = Arc::new(DummyClient::default());
    let user_2_socket = Arc::new(DummyClient::default());

    /* #region Auth */
    auth_manager.send(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
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
        request_id: None,
        auth: Arc::new(None),
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
        request_id: None,
        auth: user_1_auth.clone(),
        name: None,
        description: None,
        join_request: true,
        access_type: general::model::CreateRoomAccessType::Public,
        max_user: 4,
        metas: None,
        tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()],
        socket:user_1_socket.clone()
    }).await??;
    
    let room_1_id: RoomCreated = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let room_1_id = room_1_id.room;

    room_manager.send(JoinToRoomRequest {
        request_id: None,
        auth: user_2_auth.clone(),
        room: room_1_id,
        room_user_type: RoomUserType::User,
        socket:user_2_socket.clone()
    }).await??;

    let message: NewRoomJoinRequest = serde_json::from_str(&user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(&message.class_type[..], "NewJoinRequest");
    assert_eq!(&message.room, &room_1_id);
    assert_eq!(&message.user, user_2_auth_jwt.id.deref());
    assert_eq!(message.user_type, RoomUserType::User);

    let message: JoinRequested = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(&message.class_type[..], "JoinRequested");
    assert_eq!(&message.room, &room_1_id);

    // Get waiting list
    room_manager.send(WaitingRoomJoins {
        request_id: None,
        auth: user_1_auth.clone(),
        room: room_1_id,
        socket:user_1_socket.clone()
    }).await??;

    let message: WaitingRoomJoinsResponse = serde_json::from_str(&user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(&message.class_type[..], "WaitingRoomJoins");
    assert_eq!(&message.room, &room_1_id);
    assert_eq!(&message.users, &HashMap::from([(user_2_auth_jwt.id.deref().clone(), RoomUserType::User)]));

    // Decline waiting user
    room_manager.send(ProcessWaitingUser {
        request_id: None,
        auth: user_1_auth.clone(),
        room: room_1_id,
        user: user_2_auth_jwt.id.deref().clone(),
        status: false,
        socket: user_1_socket.clone()
    }).await??;

    let message: GenericAnswer<JoinRequestDeclined> = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    let message = message.result;
    assert_eq!(&message.class_type[..], "JoinRequestDeclined");
    assert_eq!(&message.room, &room_1_id);

    let message: Answer = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert!(message.status);
    
    /* #endregion */

    return Ok(());
}

#[actix::test]
async fn user_ban_test() -> anyhow::Result<()> {

    let config = ::general::config::get_configuration();
    let connection = create_connection(":memory:")?;

    #[cfg(feature = "stateless")]
    let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();
    let states = YummyState::new(config.clone(), #[cfg(feature = "stateless")] conn.clone());
    let executer = Arc::new(PluginExecuter::default());

    ConnectionManager::new(config.clone(), states.clone(), executer.clone(), #[cfg(feature = "stateless")] conn.clone()).start();

    create_database(&mut connection.clone().get()?)?;

    let auth_manager = AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone()), executer.clone()).start();
    let room_manager = RoomManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection), executer.clone()).start();

    let user_1_socket = Arc::new(DummyClient::default());
    let user_2_socket = Arc::new(DummyClient::default());

    /* #region Auth */
    auth_manager.send(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
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
        request_id: None,
        auth: Arc::new(None),
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
        request_id: None,
        auth: user_1_auth.clone(),
        name: None,
        description: None,
        join_request: false,
        access_type: general::model::CreateRoomAccessType::Public,
        max_user: 4,
        metas: None,
        tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()],
        socket:user_1_socket.clone()
    }).await??;
    
    let room_1_id: RoomCreated = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let room_1_id = room_1_id.room;

    room_manager.send(JoinToRoomRequest {
        request_id: None,
        auth: user_2_auth.clone(),
        room: room_1_id,
        room_user_type: RoomUserType::User,
        socket:user_2_socket.clone()
    }).await??;

    let message: Joined = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(&message.class_type[..], "Joined");

    // Not enough permission to ban user
    room_manager.send(KickUserFromRoom {
        request_id: None,
        auth: user_2_auth.clone(),
        room: room_1_id,
        user: user_1_auth_jwt.id.deref().clone(),
        socket:user_2_socket.clone(),
        ban: true
    }).await?.unwrap_err();

    let response: ReceiveError = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert!(!response.status);

    // Room owner ban the user
    room_manager.send(KickUserFromRoom {
        request_id: None,
        auth: user_1_auth.clone(),
        room: room_1_id,
        user: user_2_auth_jwt.id.deref().clone(),
        socket:user_1_socket.clone(),
        ban: true
    }).await??;

    let message: DisconnectedFromRoom = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(&message.class_type[..], "DisconnectedFromRoom");
    
    
    // User try to connect room again, but it should be failed
    room_manager.send(JoinToRoomRequest {
        request_id: None,
        auth: user_2_auth.clone(),
        room: room_1_id,
        room_user_type: RoomUserType::User,
        socket:user_2_socket.clone()
    }).await?.unwrap_err();

    let response: ReceiveError = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert!(!response.status);
    
    /* #endregion */

    return Ok(());
}

#[actix::test]
async fn kick_ban_test() -> anyhow::Result<()> {

    let config = ::general::config::get_configuration();
    let connection = create_connection(":memory:")?;

    #[cfg(feature = "stateless")]
    let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();
    let states = YummyState::new(config.clone(), #[cfg(feature = "stateless")] conn.clone());
    let executer = Arc::new(PluginExecuter::default());

    ConnectionManager::new(config.clone(), states.clone(), executer.clone(), #[cfg(feature = "stateless")] conn.clone()).start();

    create_database(&mut connection.clone().get()?)?;

    let auth_manager = AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone()), executer.clone()).start();
    let room_manager = RoomManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection), executer.clone()).start();

    let user_1_socket = Arc::new(DummyClient::default());
    let user_2_socket = Arc::new(DummyClient::default());

    /* #region Auth */
    auth_manager.send(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
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
        request_id: None,
        auth: Arc::new(None),
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
        request_id: None,
        auth: user_1_auth.clone(),
        name: None,
        description: None,
        join_request: false,
        access_type: general::model::CreateRoomAccessType::Public,
        max_user: 4,
        metas: None,
        tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()],
        socket:user_1_socket.clone()
    }).await??;
    
    let room_1_id: RoomCreated = user_1_socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let room_1_id = room_1_id.room;

    room_manager.send(JoinToRoomRequest {
        request_id: None,
        auth: user_2_auth.clone(),
        room: room_1_id,
        room_user_type: RoomUserType::User,
        socket:user_2_socket.clone()
    }).await??;

    let message: Joined = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(&message.class_type[..], "Joined");

    // Not enough permission to ban user
    room_manager.send(KickUserFromRoom {
        request_id: None,
        auth: user_2_auth.clone(),
        room: room_1_id,
        user: user_1_auth_jwt.id.deref().clone(),
        socket:user_2_socket.clone(),
        ban: false
    }).await?.unwrap_err();

    let response: ReceiveError = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert!(!response.status);

    // Room owner ban the user
    room_manager.send(KickUserFromRoom {
        request_id: None,
        auth: user_1_auth.clone(),
        room: room_1_id,
        user: user_2_auth_jwt.id.deref().clone(),
        socket:user_1_socket.clone(),
        ban: false
    }).await??;

    let message: DisconnectedFromRoom = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(&message.class_type[..], "DisconnectedFromRoom");
    
    // User try to connect room again, but it should be failed
    room_manager.send(JoinToRoomRequest {
        request_id: None,
        auth: user_2_auth.clone(),
        room: room_1_id,
        room_user_type: RoomUserType::User,
        socket:user_2_socket.clone()
    }).await??;

    let message: Joined = serde_json::from_str(&user_2_socket.clone().messages.lock().unwrap().pop_back().unwrap()).unwrap();
    assert_eq!(&message.class_type[..], "Joined");
    
    /* #endregion */

    return Ok(());
}
