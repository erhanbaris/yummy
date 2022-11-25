pub(crate) mod http;
pub(crate) mod request;
pub(crate) mod websocket;

use std::sync::Arc;

use actix::Addr;
use database::DatabaseTrait;
use general::auth::UserAuth;
use manager::{api::{auth::AuthManager, user::UserManager}, response::Response};
use manager::api::auth::model::*;
use manager::api::user::model::*;

use validator::Validate;

use crate::api::request::{AuthType, UserType};

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
pub(crate) async fn process_auth<DB: DatabaseTrait + Unpin + 'static>(auth_type: AuthType, auth_manager: Addr<AuthManager<DB>>) -> anyhow::Result<Response> {
    match auth_type {
        AuthType::Email { email, password, if_not_exist_create } => as_response!(auth_manager, EmailAuthRequest { email, password, if_not_exist_create }),
        AuthType::DeviceId { id } => as_response!(auth_manager, DeviceIdAuthRequest::new(id)),
        AuthType::CustomId { id } => as_response!(auth_manager, CustomIdAuthRequest::new(id)),
        AuthType::Refresh { token } => as_response!(auth_manager, RefreshTokenRequest { token }),
        AuthType::Restore { token } => as_response!(auth_manager, RestoreTokenRequest { token }),
        AuthType::StartUserTimeout { session_id } => as_response!(auth_manager, StartUserTimeout { session_id }),
    }
}

#[tracing::instrument(name="process_user", skip(user_manager))]
pub(crate) async fn process_user<DB: DatabaseTrait + Unpin + 'static>(user_type: UserType, user_manager: Addr<UserManager<DB>>, user: Arc<Option<UserAuth>>) -> anyhow::Result<Response> {
     match user_type {
        UserType::Me => as_response!(user_manager, GetDetailedUserInfo { user }),
        UserType::Get { user: user_id } => as_response!(user_manager, GetPublicUserInfo { user, target_user: user_id }),
        UserType::Update { name, email, password, device_id, custom_id } => as_response!(user_manager, UpdateUser { user, name, email, password, device_id, custom_id }),
    }
}
