use std::{fmt::Debug, sync::Arc, collections::HashMap};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use general::{client::ClientTrait, meta::MetaAction, model::UserInformationModel};
use actix::prelude::Message;
use validator::Validate;
use testing::client::DummyClient;

use general::{model::{UserId, UserType}, auth::UserAuth, meta::{MetaType, UserMetaAccess}};

#[derive(Message, Validate, Clone, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct GetUserInformation {
    pub request_id: Option<usize>,
    pub query: GetUserInformationEnum,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

impl GetUserInformation {
    pub fn me(request_id: Option<usize>, me: Arc<Option<UserAuth>>, socket: Arc<dyn ClientTrait + Sync + Send>) -> Self {
        Self {
            request_id,
            query: GetUserInformationEnum::Me(me),
            socket
        }
    }
    pub fn user(request_id: Option<usize>, user: UserId, requester: Arc<Option<UserAuth>>, socket: Arc<dyn ClientTrait + Sync + Send>) -> Self {
        Self {
            request_id,
            query: GetUserInformationEnum::User {
                user,
                requester
            },
            socket
        }
    }
    pub fn user_via_system(request_id: Option<usize>, user: UserId, socket: Arc<dyn ClientTrait + Sync + Send>) -> Self {
        Self {
            request_id,
            query: GetUserInformationEnum::UserViaSystem(user),
            socket
        }
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub enum GetUserInformationEnum {
    Me(Arc<Option<UserAuth>>),
    UserViaSystem(UserId),
    User { user: UserId, requester: Arc<Option<UserAuth>> }
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct UpdateUser {
    pub request_id: Option<usize>,
    pub auth: Arc<Option<UserAuth>>,
    pub target_user_id: Option<UserId>,
    pub name: Option<String>,
    pub socket: Arc<dyn ClientTrait + Sync + Send>,

    #[validate(email)]
    pub email: Option<String>,
    pub password: Option<String>,
    pub device_id: Option<String>,
    pub custom_id: Option<String>,
    pub user_type: Option<UserType>,
    pub metas: Option<HashMap<String, MetaType<UserMetaAccess>>>,
    pub meta_action: Option<MetaAction>
}

#[cfg(test)]
impl Default for UpdateUser
{
    fn default() -> Self {
        Self {
            request_id: None,
            auth: Arc::new(None),
            target_user_id: None,
            name: None,
            socket: Arc::new(DummyClient::default()),
            email: None,
            password: None,
            device_id: None,
            custom_id: None,
            metas: None,
            meta_action: None,
            user_type: None,
        }
    }
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
    MetaLimitOverToMaximum,

    #[error("User not belong to room")]
    UserNotBelongToRoom,

    #[error("'{0}' meta access level cannot be bigger than users access level")]
    MetaAccessLevelCannotBeBiggerThanUsersAccessLevel(String)
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum UserResponse {
    UserInfo {
        #[serde(flatten)]
        user: UserInformationModel
    },
}