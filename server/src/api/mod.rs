pub(crate) mod request;
pub(crate) mod websocket;

//#[cfg(test)]
mod test;

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
pub(crate) fn process_auth<DB: DatabaseTrait + Unpin + 'static>(auth_type: RequestAuthType, auth_manager: Addr<AuthManager<DB>>, auth: Arc<Option<UserAuth>>, socket: Arc<dyn ClientTrait + Sync + Send>) -> anyhow::Result<()> {
    match auth_type {
        RequestAuthType::Email { email, password, if_not_exist_create } => as_response!(auth_manager, EmailAuthRequest { auth, email, password, if_not_exist_create, socket }),
        RequestAuthType::DeviceId { id } => as_response!(auth_manager, DeviceIdAuthRequest::new(auth, id, socket)),
        RequestAuthType::CustomId { id } => as_response!(auth_manager, CustomIdAuthRequest::new(auth, id, socket)),
        RequestAuthType::Refresh { token } => as_response!(auth_manager, RefreshTokenRequest { auth, token, socket }),
        RequestAuthType::Restore { token } => as_response!(auth_manager, RestoreTokenRequest { auth, token, socket }),
        RequestAuthType::Logout => as_response!(auth_manager, LogoutRequest { auth, socket }),
    };
    Ok(())
}

#[tracing::instrument(name="process_user", skip(user_manager))]
pub(crate) fn process_user<DB: DatabaseTrait + Unpin + 'static>(user_type: RequestUserType, user_manager: Addr<UserManager<DB>>, auth: Arc<Option<UserAuth>>, socket: Arc<dyn ClientTrait + Sync + Send>) -> anyhow::Result<()> {
     match user_type {
        RequestUserType::Me => as_response!(user_manager, GetUserInformation::me(auth, socket)),
        RequestUserType::Get { user } => as_response!(user_manager, GetUserInformation::user(user, auth, socket)),
        RequestUserType::Update { name, email, password, device_id, custom_id, user_type, meta, meta_action } => as_response!(user_manager, UpdateUser { auth, name, email, password, device_id, custom_id, meta, meta_action, user_type, socket, target_user_id: None }),
    };
    Ok(())
}

#[tracing::instrument(name="process_room", skip(room_manager))]
pub(crate) fn process_room<DB: DatabaseTrait + Unpin + 'static>(room_type: RequestRoomType, room_manager: Addr<RoomManager<DB>>, auth: Arc<Option<UserAuth>>, socket: Arc<dyn ClientTrait + Sync + Send>) -> anyhow::Result<()> {
    match room_type {
        RequestRoomType::Create { name, description, access_type, max_user, tags, metas, join_request } => as_response!(room_manager, CreateRoomRequest { auth, socket, name, description, access_type, max_user, tags, metas, join_request }),
        RequestRoomType::Join { room, room_user_type } => as_response!(room_manager, JoinToRoomRequest { auth, socket, room, room_user_type }),
        RequestRoomType::Disconnect { room } => as_response!(room_manager, DisconnectFromRoomRequest { auth, socket, room }),
        RequestRoomType::Message { room, message } => as_response!(room_manager, MessageToRoomRequest { auth, socket, room, message }),
        RequestRoomType::Update { room, user_permission, name, description, max_user, join_request, metas, meta_action, access_type, tags } => as_response!(room_manager, UpdateRoom { auth, socket, room_id: room , user_permission, name, description, max_user, metas, meta_action, access_type, join_request, tags }),
        RequestRoomType::Kick { room, user } => as_response!(room_manager, KickUserFromRoom { auth, socket, room, user, ban: false }),
        RequestRoomType::Ban { room, user } => as_response!(room_manager, KickUserFromRoom { auth, socket, room, user, ban: true }),
    };
    Ok(())
}
