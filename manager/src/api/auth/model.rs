use std::{fmt::Debug, sync::Arc};
use actix::prelude::Message;
use general::{model::{SessionId, UserId}, auth::UserAuth};
use thiserror::Error;
use validator::Validate;
use general::client::ClientTrait;


#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct EmailAuthRequest {
    #[validate(email(message="Email address is not valid"))]
    pub email: String,

    #[validate(length(min = 3, max = 32, message = "Length should be between 3 to 32 chars"))]
    pub password: String,

    pub if_not_exist_create: bool,

    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct RefreshTokenRequest {

    #[validate(length(min = 275, max = 1024, message = "Length should be between 275 to 1024 chars"))]
    pub token: String,

    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct RestoreTokenRequest {

    #[validate(length(min = 275, max = 1024, message = "Length should be between 275 to 1024 chars"))]
    pub token: String,

    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct LogoutRequest {
    pub user: Arc<Option<UserAuth>>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct StartUserTimeout {
    pub session_id: SessionId
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct StopUserTimeout {
    pub session_id: SessionId
}

#[derive(Message, Debug, Validate)]
#[rtype(result = "anyhow::Result<()>")]
pub struct DeviceIdAuthRequest {
    #[validate(length(min = 8, max = 128, message = "Length should be between 8 to 128 chars"))]
    pub id: String,

    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

impl DeviceIdAuthRequest {
    pub fn new(id: String, socket: Arc<dyn ClientTrait + Sync + Send>) -> Self {
        Self { id, socket }
    }
}

#[derive(Message, Debug, Validate)]
#[rtype(result = "anyhow::Result<()>")]
pub struct CustomIdAuthRequest {
    #[validate(length(min = 8, max = 128, message = "Length should be between 8 to 128 chars"))]
    pub id: String,

    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

impl CustomIdAuthRequest {
    pub fn new(id: String, socket: Arc<dyn ClientTrait + Sync + Send>) -> Self {
        Self { id, socket }
    }
}

#[derive(Message, Validate, Debug)]
#[derive(Clone)]
#[rtype(result = "()")]
pub struct UserDisconnectRequest {
    pub user_id: UserId,
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Email and/or password not valid")]
    EmailOrPasswordNotValid,

    #[error("Session token could not generated")]
    TokenCouldNotGenerated,
    
    #[error("User token is not valid")]
    TokenNotValid,

    #[error("Only one connection allowed per user")]
    OnlyOneConnectionAllowedPerUser
}