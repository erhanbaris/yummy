pub(crate) mod http;
pub(crate) mod request;

use actix::Addr;
use database::DatabaseTrait;
use general::{web::GenericAnswer, auth::UserAuth, model::UserId};
use manager::api::{auth::{AuthManager, RefreshTokenRequest, DeviceIdAuthRequest, EmailAuthRequest, CustomIdAuthRequest, AuthError, RestoreTokenRequest}, user::{UserManager, GetDetailedUserInfo, GetPublicUserInfo, UpdateUser}};
use validator::Validate;

use crate::api::request::{AuthType, UserType};

macro_rules! as_error {
    ($error: expr) => {
        serde_json::to_string(&GenericAnswer {
            status: false,
            result: Some($error.to_string()),
        })
    }
}

macro_rules! as_ok {
    ($result: expr) => {
        serde_json::to_string(&GenericAnswer {
            status: true,
            result: Some($result),
        })
    }
}

macro_rules! as_response {
    ($manager: expr, $message: expr) => {
        {
            let message = $message;
            match message.validate() {
                Ok(_) => match $manager.send(message).await {
                    Ok(actix_result) => match actix_result {
                        Ok(result) => as_ok!(result),
                        Err(error) => as_error!(error)
                    },
                    Err(error) => as_error!(error)
                },
                Err(error) => as_error!(error)
            }
        }
    };
}

pub(crate) async fn process_auth<DB: DatabaseTrait + Unpin + 'static>(auth_type: AuthType, auth_manager: Addr<AuthManager<DB>>) -> Result<String, serde_json::Error> {
    match auth_type {
        AuthType::Email { email, password, if_not_exist_create } => as_response!(auth_manager, EmailAuthRequest { email, password, if_not_exist_create }),
        AuthType::DeviceId { id } => as_response!(auth_manager, DeviceIdAuthRequest::new(id)),
        AuthType::CustomId { id } => as_response!(auth_manager, CustomIdAuthRequest::new(id)),
        AuthType::Refresh { token } => as_response!(auth_manager, RefreshTokenRequest { token }),
        AuthType::Restore { token } => as_response!(auth_manager, RestoreTokenRequest { token }),
    }
}

pub(crate) async fn process_user<DB: DatabaseTrait + Unpin + 'static>(user_type: UserType, user_manager: Addr<UserManager<DB>>, user: &Option<UserAuth>) -> Result<String, serde_json::Error> {
     match user_type {
        UserType::Me => match user {
            Some(auth) => as_response!(user_manager, GetDetailedUserInfo { user: auth.user }),
            None => as_error!(AuthError::TokenNotValid)
        },
        UserType::Get { user } => as_response!(user_manager, GetPublicUserInfo { user: UserId(user) }),
        UserType::Update { name, email, password, device_id, custom_id } => match user {
            Some(auth) => as_response!(user_manager, UpdateUser { user: auth.user, name, email, password, device_id, custom_id }),
            None => as_error!(AuthError::TokenNotValid)
        },
    }
}
