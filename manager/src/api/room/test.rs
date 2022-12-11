use actix::Recipient;
use general::model::WebsocketMessage;
use uuid::Uuid;

use general::auth::UserAuth;
use general::auth::validate_auth;
use general::config::YummyConfig;
use general::config::get_configuration;
use general::model::YummyState;
use std::env::temp_dir;
use std::sync::Arc;

use actix::Actor;
use actix::Addr;
use anyhow::Ok;
use database::{create_database, create_connection};

use super::*;
use crate::api::auth::AuthManager;
use crate::api::auth::model::*;

use actix::{Context, Handler};
struct dummy_actor;

impl Actor for dummy_actor {
    type Context = Context<Self>;
}

impl Handler<WebsocketMessage> for dummy_actor {
    type Result = ();
    
    fn handle(&mut self, _: WebsocketMessage, _: &mut Self::Context) { }
}

macro_rules! email_auth {
    ($auth_manager: expr, $config: expr, $email: expr, $password: expr, $create: expr, $recipient: expr) => {
        {
            let token = $auth_manager.send(EmailAuthRequest {
                email: $email,
                password: $password,
                if_not_exist_create: $create,
                socket: $recipient.clone()
            }).await??;
        
            let token = match token {
                Response::Auth(token, _) => token,
                _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
            };
        
            let user_jwt = validate_auth($config, token).unwrap().user;
            Arc::new(Some(UserAuth {
                user: user_jwt.id,
                session: user_jwt.session
            }))
        }
    };
}


fn create_actor() -> anyhow::Result<(Addr<RoomManager<database::SqliteStore>>, Addr<AuthManager<database::SqliteStore>>, Arc<YummyConfig>, Arc<YummyState>, Recipient<WebsocketMessage>)> {
    let mut db_location = temp_dir();
    db_location.push(format!("{}.db", Uuid::new_v4()));
    
    let config = get_configuration();
    let states = Arc::new(YummyState::default());
    let communication_manager = CommunicationManager::new(config.clone(), states.clone()).start();
    let connection = create_connection(db_location.to_str().unwrap())?;
    create_database(&mut connection.clone().get()?)?;
    Ok((RoomManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone()), communication_manager.recipient::<SendMessage>()).start(), AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection)).start(), config, states.clone(), dummy_actor {}.start().recipient()))
}

#[actix::test]
async fn create_room_1() -> anyhow::Result<()> {
    let (room_manager, auth_manager, config, states, recipient) = create_actor()?;
    let user = email_auth!(auth_manager, config.clone(), "user@gmail.com".to_string(), "erhan".to_string(), true, recipient);

    let response = room_manager.send(CreateRoomRequest {
        user: user.clone(),
        disconnect_from_other_room: false,
        name: None,
        access_type: general::model::CreateRoomAccessType::Friend,
        max_user: 4,
        tags: Vec::new()
    }).await??;

    let room_id = match response {
        Response::RoomInformation(room_id) => room_id,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::RoomInformation'")); }
    };

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
    let (room_manager, auth_manager, config, states, recipient) = create_actor()?;
    let user = email_auth!(auth_manager, config.clone(), "user@gmail.com".to_string(), "erhan".to_string(), true, recipient);

    let response = room_manager.send(CreateRoomRequest {
        user: user.clone(),
        disconnect_from_other_room: false,
        name: None,
        access_type: general::model::CreateRoomAccessType::Tag("123456".to_string()),
        max_user: 4,
        tags: vec!["tag 1".to_string(), "tag 2".to_string(), "tag 3".to_string(), "tag 4".to_string()]
    }).await??;

    let room_id = match response {
        Response::RoomInformation(room_id) => room_id,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::RoomInformation'")); }
    };

    let user_id = match user.as_ref() {
        Some(user) => user.user.clone(),
        None => return Err(anyhow::anyhow!("UserId not found"))
    };

    assert!(room_id.get() != uuid::Uuid::nil());
    assert!(states.get_user_room(user_id).is_some());
    
    Ok(())
}
