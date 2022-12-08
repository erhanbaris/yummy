pub(crate) mod request;
pub(crate) mod websocket;

use std::sync::Arc;

use actix::{Addr, Recipient};
use database::DatabaseTrait;
use general::{auth::UserAuth, meta::MetaAccess, model::WebsocketMessage};
use manager::{api::{auth::AuthManager, user::UserManager, room::RoomManager}, response::Response};
use manager::api::auth::model::*;
use manager::api::user::model::*;

use validator::Validate;

use crate::api::request::{RequestAuthType, RequestUserType};

use self::request::RequestRoomType;

macro_rules! as_response {
    ($manager: expr, $message: expr) => {
        {
            let message = $message;
            match message.validate() {
                Ok(_) => $manager.send(message).await?,
                Err(error) => Err(anyhow::anyhow!(error))
            }
        }
    }
}

#[tracing::instrument(name="process_auth", skip(auth_manager))]
pub(crate) async fn process_auth<DB: DatabaseTrait + Unpin + 'static>(auth_type: RequestAuthType, auth_manager: Addr<AuthManager<DB>>, me: Arc<Option<UserAuth>>, socket: Recipient<WebsocketMessage>) -> anyhow::Result<Response> {
    match auth_type {
        RequestAuthType::Email { email, password, if_not_exist_create } => as_response!(auth_manager, EmailAuthRequest { email, password, if_not_exist_create, socket }),
        RequestAuthType::DeviceId { id } => as_response!(auth_manager, DeviceIdAuthRequest::new(id, socket)),
        RequestAuthType::CustomId { id } => as_response!(auth_manager, CustomIdAuthRequest::new(id, socket)),
        RequestAuthType::Refresh { token } => as_response!(auth_manager, RefreshTokenRequest { token, socket }),
        RequestAuthType::Restore { token } => as_response!(auth_manager, RestoreTokenRequest { token, socket }),
        RequestAuthType::Logout => as_response!(auth_manager, LogoutRequest { user: me }),
    }
}

#[tracing::instrument(name="process_user", skip(user_manager))]
pub(crate) async fn process_user<DB: DatabaseTrait + Unpin + 'static>(user_type: RequestUserType, user_manager: Addr<UserManager<DB>>, me: Arc<Option<UserAuth>>) -> anyhow::Result<Response> {
     match user_type {
        RequestUserType::Me => as_response!(user_manager, GetUserInformation::me(me)),
        RequestUserType::Get { user } => as_response!(user_manager, GetUserInformation::user(user, me)),
        RequestUserType::Update { name, email, password, device_id, custom_id, user_type, meta } => as_response!(user_manager, UpdateUser { user: me, name, email, password, device_id, custom_id, meta, user_type, access_level: MetaAccess::Me }),
    }
}

#[tracing::instrument(name="process_room", skip(room_manager))]
pub(crate) async fn process_room<DB: DatabaseTrait + Unpin + 'static>(room_type: RequestRoomType, room_manager: Addr<RoomManager<DB>>, me: Arc<Option<UserAuth>>) -> anyhow::Result<Response> {
     match room_type {
        RequestRoomType::Create {  } => todo!(),
    }
}
