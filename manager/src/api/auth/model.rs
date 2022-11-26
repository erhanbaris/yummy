use std::{fmt::Debug, sync::Arc};
use actix::prelude::Message;
use general::{model::SessionId, auth::UserAuth};
use thiserror::Error;
use validator::Validate;

use crate::response::Response;

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<Response>")]
pub struct EmailAuthRequest {
    #[validate(email(message="Email address is not valid"))]
    pub email: String,

    #[validate(length(min = 3, max = 32, message = "Length should be between 3 to 32 chars"))]
    pub password: String,

    pub if_not_exist_create: bool,
}

unsafe impl Send for EmailAuthRequest {}
unsafe impl Sync for EmailAuthRequest {}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<Response>")]
pub struct RefreshTokenRequest {

    #[validate(length(min = 275, max = 1024, message = "Length should be between 275 to 1024 chars"))]
    pub token: String
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<Response>")]
pub struct RestoreTokenRequest {

    #[validate(length(min = 275, max = 1024, message = "Length should be between 275 to 1024 chars"))]
    pub token: String
}

unsafe impl Send for RefreshTokenRequest {}
unsafe impl Sync for RefreshTokenRequest {}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<Response>")]
pub struct LogoutRequest {
    pub user: Arc<Option<UserAuth>>
}

unsafe impl Send for LogoutRequest {}
unsafe impl Sync for LogoutRequest {}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<Response>")]
pub struct StartUserTimeout {
    pub session_id: SessionId
}

unsafe impl Send for StartUserTimeout {}
unsafe impl Sync for StartUserTimeout {}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<Response>")]
pub struct StopUserTimeout {
    pub session_id: SessionId
}

unsafe impl Send for StopUserTimeout {}
unsafe impl Sync for StopUserTimeout {}

#[derive(Message, Debug, Validate)]
#[rtype(result = "anyhow::Result<Response>")]
pub struct DeviceIdAuthRequest {
    #[validate(length(min = 8, max = 128, message = "Length should be between 8 to 128 chars"))]
    pub id: String
}

impl DeviceIdAuthRequest {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

unsafe impl Send for DeviceIdAuthRequest {}
unsafe impl Sync for DeviceIdAuthRequest {}

#[derive(Message, Debug, Validate)]
#[rtype(result = "anyhow::Result<Response>")]
pub struct CustomIdAuthRequest {
    #[validate(length(min = 8, max = 128, message = "Length should be between 3 to 128 chars"))]
    pub id: String
}

impl CustomIdAuthRequest {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

unsafe impl Send for CustomIdAuthRequest {}
unsafe impl Sync for CustomIdAuthRequest {}

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