pub mod request;
pub mod websocket;

#[cfg(test)]
mod test;

use std::sync::Arc;

use actix::Addr;
use yummy_database::DatabaseTrait;
use yummy_general::client::ClientTrait;
use yummy_model::auth::UserAuth;
use yummy_manager::auth::AuthManager;
use yummy_manager::room::RoomManager;
use yummy_manager::user::UserManager;
use yummy_manager::auth::model::*;
use yummy_manager::user::model::*;
use yummy_manager::room::model::*;

use validator::{Validate, ValidationErrors};

use crate::api::request::{RequestAuthType, RequestUserType};

use self::request::RequestRoomType;

macro_rules! as_response {
    ($request_id: expr, $manager: expr, $message: expr) => {
        {
            let message = $message;
            match message.validate() {
                Ok(_) => $manager.do_send(message),
                Err(error) => return Err(($request_id, error))
            }
        }
    }
}

type ProcessResult = Result<(), (Option<usize>, ValidationErrors)>;

#[tracing::instrument(name="process_auth", skip(auth_manager))]
pub(crate) fn process_auth<DB: DatabaseTrait + Unpin + 'static>(request_id: Option<usize>, auth_type: RequestAuthType, auth_manager: Addr<AuthManager<DB>>, auth: Arc<Option<UserAuth>>, socket: Arc<dyn ClientTrait + Sync + Send>) -> ProcessResult {
    
    match auth_type {
        RequestAuthType::Email { email, password, if_not_exist_create } => as_response!(request_id, auth_manager, EmailAuthRequest { request_id, auth, email, password, if_not_exist_create, socket }),
        RequestAuthType::DeviceId { id } => as_response!(request_id, auth_manager, DeviceIdAuthRequest::new(request_id, auth, id, socket)),
        RequestAuthType::CustomId { id } => as_response!(request_id, auth_manager, CustomIdAuthRequest::new(request_id, auth, id, socket)),
        RequestAuthType::Refresh { token } => as_response!(request_id, auth_manager, RefreshTokenRequest { request_id, auth, token, socket }),
        RequestAuthType::Restore { token } => as_response!(request_id, auth_manager, RestoreTokenRequest { request_id, auth, token, socket }),
        RequestAuthType::Logout => as_response!(request_id, auth_manager, LogoutRequest { request_id, auth, socket }),
    };
    Ok(())
}

#[tracing::instrument(name="process_user", skip(user_manager))]
pub(crate) fn process_user<DB: DatabaseTrait + Unpin + 'static>(request_id: Option<usize>, user_type: RequestUserType, user_manager: Addr<UserManager<DB>>, auth: Arc<Option<UserAuth>>, socket: Arc<dyn ClientTrait + Sync + Send>) -> ProcessResult {
     match user_type {
        RequestUserType::Me => as_response!(request_id, user_manager, GetUserInformation::me(request_id, auth, socket)),
        RequestUserType::Get { user_id } => as_response!(request_id, user_manager, GetUserInformation::user(request_id, user_id, auth, socket)),
        RequestUserType::Update { name, email, password, device_id, custom_id, user_type, metas, meta_action } => as_response!(request_id, user_manager, UpdateUser { request_id, auth, name, email, password, device_id, custom_id, metas, meta_action, user_type, socket, target_user_id: None }),
    };
    Ok(())
}

#[tracing::instrument(name="process_room", skip(room_manager))]
pub(crate) fn process_room<DB: DatabaseTrait + Unpin + 'static>(request_id: Option<usize>, room_type: RequestRoomType, room_manager: Addr<RoomManager<DB>>, auth: Arc<Option<UserAuth>>, socket: Arc<dyn ClientTrait + Sync + Send>) -> ProcessResult {
    match room_type {
        RequestRoomType::Create { name, description, access_type, max_user, tags, metas, join_request } => as_response!(request_id, room_manager, CreateRoomRequest { request_id, auth, socket, name, description, access_type, max_user, tags, metas, join_request }),
        RequestRoomType::Join { room_id, room_user_type } => as_response!(request_id, room_manager, JoinToRoomRequest { request_id, auth, socket, room_id, room_user_type }),
        RequestRoomType::Disconnect { room_id } => as_response!(request_id, room_manager, DisconnectFromRoomRequest { request_id, auth, socket, room_id }),
        RequestRoomType::Message { room_id, message } => as_response!(request_id, room_manager, MessageToRoomRequest { request_id, auth, socket, room_id, message }),
        RequestRoomType::Update { room_id, user_permission, name, description, max_user, join_request, metas, meta_action, access_type, tags } => as_response!(request_id, room_manager, UpdateRoom { request_id, auth, socket, room_id , user_permission, name, description, max_user, metas, meta_action, access_type, join_request, tags }),
        RequestRoomType::Kick { room_id, user_id } => as_response!(request_id, room_manager, KickUserFromRoom { request_id, auth, socket, room_id, user_id, ban: false }),
        RequestRoomType::Ban { room_id, user_id } => as_response!(request_id, room_manager, KickUserFromRoom { request_id, auth, socket, room_id, user_id, ban: true }),
    };
    Ok(())
}
