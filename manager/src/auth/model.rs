use std::{fmt::Debug, sync::Arc};
use actix::prelude::Message;
use general::{model::{SessionId, UserId}, auth::UserAuth, password::Password};
use serde::Serialize;
use thiserror::Error;
use validator::{Validate, ValidationError};
use general::client::ClientTrait;

fn validate_unique_password(pass: &Password) -> Result<(), ValidationError> {
    let pass = pass.get();
    if pass.len() > 32 || pass.len() < 3 {
        return Err(ValidationError::new("Length should be between 3 to 32 chars"));
    }

    Ok(())
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct EmailAuthRequest {
    pub user: Arc<Option<UserAuth>>,

    #[validate(email(message="Email address is not valid"))]
    pub email: String,

    #[validate(custom(function="validate_unique_password", message="Length should be between 3 to 32 chars"))]
    pub password: Password,

    pub if_not_exist_create: bool,

    pub socket: Arc<dyn ClientTrait + Sync + Send>,
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct RefreshTokenRequest {
    pub user: Arc<Option<UserAuth>>,

    #[validate(length(min = 275, max = 1024, message = "Length should be between 275 to 1024 chars"))]
    pub token: String,

    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct RestoreTokenRequest {
    pub user: Arc<Option<UserAuth>>,

    #[validate(length(min = 275, max = 1024, message = "Length should be between 275 to 1024 chars"))]
    pub token: String,

    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct LogoutRequest {
    pub user: Arc<Option<UserAuth>>,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct StartUserTimeout {
    pub user: Arc<Option<UserAuth>>,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct StopUserTimeout {
    pub session_id: SessionId
}

#[derive(Message, Debug, Validate)]
#[rtype(result = "anyhow::Result<()>")]
pub struct DeviceIdAuthRequest {
    pub user: Arc<Option<UserAuth>>,

    #[validate(length(min = 8, max = 128, message = "Length should be between 8 to 128 chars"))]
    pub id: String,

    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

impl DeviceIdAuthRequest {
    pub fn new(user: Arc<Option<UserAuth>>, id: String, socket: Arc<dyn ClientTrait + Sync + Send>) -> Self {
        Self { user, id, socket }
    }
}

#[derive(Message, Debug, Validate)]
#[rtype(result = "anyhow::Result<()>")]
pub struct CustomIdAuthRequest {
    pub user: Arc<Option<UserAuth>>,

    #[validate(length(min = 8, max = 128, message = "Length should be between 8 to 128 chars"))]
    pub id: String,

    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

impl CustomIdAuthRequest {
    pub fn new(user: Arc<Option<UserAuth>>, id: String, socket: Arc<dyn ClientTrait + Sync + Send>) -> Self {
        Self { user, id, socket }
    }
}

#[derive(Message, Validate, Debug, Clone)]
#[rtype(result = "()")]
pub struct UserDisconnect {
    pub user: Arc<Option<UserAuth>>,
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

impl<'a> From<AuthResponse> for String {
    fn from(source: AuthResponse) -> Self {
        serde_json::to_string(&source).unwrap()
    }
}
