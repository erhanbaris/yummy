pub(crate) mod http;
pub(crate) mod request;
pub(crate) mod websocket;

use std::sync::Arc;

use actix::Addr;
use database::DatabaseTrait;
use general::{auth::UserAuth, meta::MetaAccess};
use manager::{api::{auth::AuthManager, user::UserManager}, response::Response};
use manager::api::auth::model::*;
use manager::api::user::model::*;

use validator::Validate;

use crate::api::request::{RequestAuthType, RequestUserType};

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
pub(crate) async fn process_auth<DB: DatabaseTrait + Unpin + 'static>(auth_type: RequestAuthType, auth_manager: Addr<AuthManager<DB>>, user: Arc<Option<UserAuth>>) -> anyhow::Result<Response> {
    match auth_type {
        RequestAuthType::Email { email, password, if_not_exist_create } => as_response!(auth_manager, EmailAuthRequest { email, password, if_not_exist_create }),
        RequestAuthType::DeviceId { id } => as_response!(auth_manager, DeviceIdAuthRequest::new(id)),
        RequestAuthType::CustomId { id } => as_response!(auth_manager, CustomIdAuthRequest::new(id)),
        RequestAuthType::Refresh { token } => as_response!(auth_manager, RefreshTokenRequest { token }),
        RequestAuthType::Restore { token } => as_response!(auth_manager, RestoreTokenRequest { token }),
        RequestAuthType::Logout => as_response!(auth_manager, LogoutRequest { user }),
        RequestAuthType::StartUserTimeout { session_id } => as_response!(auth_manager, StartUserTimeout { session_id }),
    }
}

#[tracing::instrument(name="process_user", skip(user_manager))]
pub(crate) async fn process_user<DB: DatabaseTrait + Unpin + 'static>(user_type: RequestUserType, user_manager: Addr<UserManager<DB>>, user: Arc<Option<UserAuth>>) -> anyhow::Result<Response> {
     match user_type {
        RequestUserType::Me => as_response!(user_manager, GetUserInformation { requester_user: user, target_user: None }),
        RequestUserType::Get { user: user_id } => as_response!(user_manager, GetUserInformation { requester_user: user, target_user: Some(user_id) }),
        RequestUserType::Update { name, email, password, device_id, custom_id, user_type, meta } => as_response!(user_manager, UpdateUser { user, name, email, password, device_id, custom_id, meta, user_type, access_level: MetaAccess::Me }),
    }
}
