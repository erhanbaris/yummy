use std::{fmt::Debug, sync::Arc};
use actix::prelude::Message;
use yummy_model::{auth::UserAuth, SessionId, password::Password};
use serde::Serialize;
use thiserror::Error;
use validator::{Validate, ValidationError};
use yummy_general::client::ClientTrait;
use yummy_macros::model;

fn validate_unique_password(pass: &Password) -> Result<(), ValidationError> {
    let pass = pass.get();
    if pass.len() > 32 || pass.len() < 3 {
        return Err(ValidationError::new("Length should be between 3 to 32 chars"));
    }

    Ok(())
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
#[model(request_type="RequestAuthTypeVariant::Email")]
pub struct EmailAuthRequest {
    pub request_id: Option<usize>,
    pub auth: Arc<Option<UserAuth>>,

    #[validate(email(message="Email address is not valid"))]
    pub email: String,

    #[validate(custom(function="validate_unique_password", message="Length should be between 3 to 32 chars"))]
    pub password: Password,

    pub if_not_exist_create: bool,

    pub socket: Arc<dyn ClientTrait + Sync + Send>,
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
#[model(request_type="RequestAuthTypeVariant::Refresh")]
pub struct RefreshTokenRequest {
    pub request_id: Option<usize>,

    pub auth: Arc<Option<UserAuth>>,

    #[validate(length(min = 275, max = 1024, message = "Length should be between 275 to 1024 chars"))]
    pub token: String,

    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
#[model(request_type="RequestAuthTypeVariant::Restore")]
pub struct RestoreTokenRequest {
    pub request_id: Option<usize>,

    pub auth: Arc<Option<UserAuth>>,

    #[validate(length(min = 275, max = 1024, message = "Length should be between 275 to 1024 chars"))]
    pub token: String,

    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
#[model(request_type="RequestAuthTypeVariant::Logout")]
pub struct LogoutRequest {
    pub request_id: Option<usize>,
    pub auth: Arc<Option<UserAuth>>,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct StartUserTimeout {
    pub auth: Arc<Option<UserAuth>>,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct StopUserTimeout {
    pub session_id: SessionId
}

#[derive(Message, Debug, Validate)]
#[rtype(result = "anyhow::Result<()>")]
#[model(request_type="RequestAuthTypeVariant::AuthDeviceId")]
pub struct DeviceIdAuthRequest {
    pub request_id: Option<usize>,

    pub auth: Arc<Option<UserAuth>>,

    #[validate(length(min = 8, max = 128, message = "Length should be between 8 to 128 chars"))]
    pub id: String,

    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

impl DeviceIdAuthRequest {
    pub fn new(request_id: Option<usize>, auth: Arc<Option<UserAuth>>, id: String, socket: Arc<dyn ClientTrait + Sync + Send>) -> Self {
        Self { request_id, auth, id, socket }
    }
}

#[derive(Message, Debug, Validate)]
#[rtype(result = "anyhow::Result<()>")]
#[model(request_type="RequestAuthTypeVariant::AuthCustomId")]
pub struct CustomIdAuthRequest {
    pub request_id: Option<usize>,
    pub auth: Arc<Option<UserAuth>>,

    #[validate(length(min = 8, max = 128, message = "Length should be between 8 to 128 chars"))]
    pub id: String,

    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

impl CustomIdAuthRequest {
    pub fn new(request_id: Option<usize>, auth: Arc<Option<UserAuth>>, id: String, socket: Arc<dyn ClientTrait + Sync + Send>) -> Self {
        Self { request_id, auth, id, socket }
    }
}

#[derive(Message, Validate, Debug, Clone)]
#[rtype(result = "()")]
pub struct ConnUserDisconnect {
    pub request_id: Option<usize>,
    pub auth: Arc<Option<UserAuth>>,
    pub send_message: bool,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug, Clone)]
#[rtype(result = "()")]
pub struct RoomUserDisconnect {
    pub request_id: Option<usize>,
    pub auth: Arc<Option<UserAuth>>,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug, Clone)]
#[rtype(result = "()")]
pub struct AuthUserDisconnect {
    pub request_id: Option<usize>,
    pub auth: Arc<Option<UserAuth>>,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Email and/or password not valid")]
    EmailOrPasswordNotValid,

    #[error("Session token could not generated")]
    TokenCouldNotGenerated,
    
    #[error("User token is not valid")]
    TokenNotValid,

    #[error("User not logged in")]
    UserNotLoggedIn
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum AuthResponse {
    Authenticated { token: String }
}

