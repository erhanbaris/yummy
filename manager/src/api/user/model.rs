use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use actix::prelude::Message;
use validator::Validate;

use general::model::UserId;

use crate::response::Response;

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<Response>")]
pub struct GetDetailedUserInfo {
    pub user: UserId
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<Response>")]
pub struct GetPublicUserInfo {
    pub user: UserId
}

#[derive(Message, Validate, Debug, Default)]
#[rtype(result = "anyhow::Result<Response>")]
pub struct UpdateUser {
    pub user: UserId,
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
