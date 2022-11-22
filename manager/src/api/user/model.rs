use std::{fmt::Debug, sync::Arc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use actix::prelude::Message;
use validator::Validate;

use general::{model::UserId, auth::UserAuth};

use crate::response::Response;

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<Response>")]
pub struct GetDetailedUserInfo {
    pub user: Arc<Option<UserAuth>>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<Response>")]
pub struct GetPublicUserInfo {
    pub user: Arc<Option<UserAuth>>,
    pub target_user: UserId
}

#[derive(Message, Validate, Debug, Default)]
#[rtype(result = "anyhow::Result<Response>")]
pub struct UpdateUser {
    pub user: Arc<Option<UserAuth>>,
    pub name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub password: Option<String>,
    pub device_id: Option<String>,
    pub custom_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UpdateUserFieldType {
    #[serde(rename = "name")]
    Name,

    #[serde(rename = "password")]
    Password,

    #[serde(rename = "device_id")]
    DeviceId,

    #[serde(rename = "custom_id")]
    CustomId,

    #[serde(rename = "email")]
    Email
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserField {
    #[serde(rename = "type")]
    field: UpdateUserFieldType,
    value: Option<String>
}

#[derive(Error, Debug, PartialEq)]
pub enum UserError {
    #[error("User not found")]
    UserNotFound,

    #[error("The user's email address cannot be changed.")]
    CannotChangeEmail,

    #[error("The password is too small")]
    PasswordIsTooSmall,

    #[error("Update information missing")]
    UpdateInformationMissing
}
