pub(crate) mod request;
pub(crate) mod websocket;

use std::sync::Arc;

use actix::Addr;
use database::DatabaseTrait;
use general::{auth::UserAuth, client::ClientTrait};
use manager::auth::AuthManager;
use manager::room::RoomManager;
use manager::user::UserManager;
use manager::auth::model::*;
use manager::user::model::*;
use manager::room::model::*;

use validator::Validate;

use crate::api::request::{RequestAuthType, RequestUserType};

use self::request::RequestRoomType;

macro_rules! as_response {
    ($manager: expr, $message: expr) => {
        {
            let message = $message;
            match message.validate() {
                Ok(_) => $manager.do_send(message),
                Err(error) => return Err(anyhow::anyhow!(error))
            }
        }
    }
}

#[tracing::instrument(name="process_auth", skip(auth_manager))]
pub(crate) fn process_auth<DB: DatabaseTrait + Unpin + 'static>(auth_type: RequestAuthType, auth_manager: Addr<AuthManager<DB>>, me: Arc<Option<UserAuth>>, socket: Arc<dyn ClientTrait + Sync + Send>) -> anyhow::Result<()> {
    match auth_type {
        RequestAuthType::Email { email, password, if_not_exist_create } => as_response!(auth_manager, EmailAuthRequest { email, password, if_not_exist_create, socket }),
        RequestAuthType::DeviceId { id } => as_response!(auth_manager, DeviceIdAuthRequest::new(id, socket)),
        RequestAuthType::CustomId { id } => as_response!(auth_manager, CustomIdAuthRequest::new(id, socket)),
        RequestAuthType::Refresh { token } => as_response!(auth_manager, RefreshTokenRequest { token, socket }),
        RequestAuthType::Restore { token } => as_response!(auth_manager, RestoreTokenRequest { token, socket }),
        RequestAuthType::Logout => as_response!(auth_manager, LogoutRequest { user: me, socket }),
    };
    Ok(())
}

#[tracing::instrument(name="process_user", skip(user_manager))]
pub(crate) fn process_user<DB: DatabaseTrait + Unpin + 'static>(user_type: RequestUserType, user_manager: Addr<UserManager<DB>>, me: Arc<Option<UserAuth>>, socket: Arc<dyn ClientTrait + Sync + Send>) -> anyhow::Result<()> {
     match user_type {
        RequestUserType::Me => as_response!(user_manager, GetUserInformation::me(me, socket)),
        RequestUserType::Get { user } => as_response!(user_manager, GetUserInformation::user(user, me, socket)),
        RequestUserType::Update { name, email, password, device_id, custom_id, user_type, meta, meta_action } => as_response!(user_manager, UpdateUser { user: me, name, email, password, device_id, custom_id, meta, meta_action, user_type, socket, target_user_id: None }),
    };
    Ok(())
}

#[tracing::instrument(name="process_room", skip(room_manager))]
pub(crate) fn process_room<DB: DatabaseTrait + Unpin + 'static>(room_type: RequestRoomType, room_manager: Addr<RoomManager<DB>>, me: Arc<Option<UserAuth>>, socket: Arc<dyn ClientTrait + Sync + Send>) -> anyhow::Result<()> {
    match room_type {
        RequestRoomType::Create { disconnect_from_other_room, name, access_type, max_user, tags, meta } => as_response!(room_manager, CreateRoomRequest { user: me, socket, disconnect_from_other_room, name, access_type, max_user, tags, meta }),
        RequestRoomType::Join { room, room_user_type } => as_response!(room_manager, JoinToRoomRequest { user: me, socket, room, room_user_type }),
        RequestRoomType::Disconnect { room } => as_response!(room_manager, DisconnectFromRoomRequest { user: me, socket, room }),
        RequestRoomType::Message { room, message } => as_response!(room_manager, MessageToRoomRequest { user: me, socket, room, message }),
        RequestRoomType::Update { room, user_permission, name, max_user, meta, access_type, tags } => as_response!(room_manager, UpdateRoom { user: me, socket, room_id: room , user_permission, name, max_user, meta, access_type, tags }),
    };
    Ok(())
}
