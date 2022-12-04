use std::{fmt::Debug, sync::Arc, collections::HashMap};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use actix::prelude::Message;
use validator::Validate;

use general::{model::{UserId, UserType}, auth::UserAuth, meta::{MetaType, MetaAccess}};

use crate::response::Response;

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<Response>")]
pub struct GetUserInformation {
    pub query: GetUserInformationEnum
}

impl GetUserInformation {
    pub fn me(me: Arc<Option<UserAuth>>) -> Self {
        Self {
            query: GetUserInformationEnum::Me(me)
        }
    }
    pub fn user(user: UserId, requester: Arc<Option<UserAuth>>) -> Self {
        Self {
            query: GetUserInformationEnum::User {
                user,
                requester
            }
        }
    }
    pub fn user_via_system(user: UserId) -> Self {
        Self {
            query: GetUserInformationEnum::UserViaSystem(user)
        }
    }
}

#[derive(Debug)]
pub enum GetUserInformationEnum {
    Me(Arc<Option<UserAuth>>),
    UserViaSystem(UserId),
    User { user: UserId, requester: Arc<Option<UserAuth>> }
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
    pub user_type: Option<UserType>,
    pub meta: Option<HashMap<String, MetaType>>,
    pub access_level: MetaAccess
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

#[derive(Error, Debug, PartialEq, Eq)]
pub enum UserError {
    #[error("User not found")]
    UserNotFound,

    #[error("The user's email address cannot be changed.")]
    CannotChangeEmail,

    #[error("The password is too small")]
    PasswordIsTooSmall,

    #[error("Update information missing")]
    UpdateInformationMissing,

    #[error("Meta limit over to maximum")]
    MetaLimitOverToMaximum
}
