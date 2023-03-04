#[macro_use]
extern crate diesel;

/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* **************************************************************************************************************** */
pub mod auth;
pub mod config;
pub mod user;
pub mod meta;
pub mod web;
pub mod schema;

/* **************************************************************************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::hash::Hash;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;

use meta::collection::UserMetaCollection;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_repr::{Serialize_repr, Deserialize_repr};

use actix::MessageResponse;
use actix::prelude::Message;

use diesel::deserialize::FromSql;
use diesel::serialize::IsNull;
use diesel::*;
use diesel::serialize::{ToSql, Output};
use diesel::sql_types::*;
use diesel::expression::AsExpression;

use num_derive::FromPrimitive;
use num_derive::ToPrimitive;

#[allow(unused_imports)]
use num_traits::FromPrimitive;

use uuid::Uuid;

use crate::auth::UserJwt;
use crate::meta::{UserMetaAccess, MetaType};
use crate::web::GenericAnswer;

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* **************************************************** MACROS **************************************************** */
/* **************************************************************************************************************** */
macro_rules! generate_type {
    ($name: ident) => {
        
        #[derive(MessageResponse, Deserialize, Serialize, Eq, PartialEq, Debug, Clone, Hash, Ord, PartialOrd)]
        #[derive(AsExpression, FromSqlRow)]
        #[diesel(sql_type = Text)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self::default()
            }

            pub fn is_empty(&self) -> bool {
                self.0 == uuid::Uuid::nil()
            }

            pub fn get(&self) -> &Uuid {
                &self.0
            }

            #[allow(dead_code)]
            fn try_from(data: String) -> Result<Self, uuid::Error> {
                uuid::Uuid::parse_str(&data).map(|item| $name(item))
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self(uuid::Uuid::new_v4())
            }
        }

        impl From<String> for $name {
            fn from(data: String) -> Self {
                $name(uuid::Uuid::parse_str(&data).unwrap_or_default())
            }
        }

        impl From<Uuid> for $name {
            fn from(data: Uuid) -> Self {
                $name(data)
            }
        }

        impl ToString for $name {
            fn to_string(&self) -> String {
                self.0.to_string()
            }
        }

        #[cfg(feature = "stateless")]
        impl redis::FromRedisValue for $name {
            fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
                let result: redis::RedisResult<String> = redis::FromRedisValue::from_redis_value(v);
                match result {
                    Ok(value) => Ok($name::from(value)),
                    Err(_) => Ok($name::default())
                }
            }
        }

        impl ToSql<Text, diesel::sqlite::Sqlite> for $name where String: ToSql<Text, diesel::sqlite::Sqlite> {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, diesel::sqlite::Sqlite>) -> serialize::Result {
                out.set_value(self.get().to_string());
                Ok(IsNull::No)
            }
        }

        impl FromSql<Text, diesel::sqlite::Sqlite> for $name where String: FromSql<Text, diesel::sqlite::Sqlite> {
            fn from_sql(bytes: backend::RawValue<diesel::sqlite::Sqlite>) -> deserialize::Result<Self> {
                let value = String::from_utf8(<Vec<u8>>::from_sql(bytes)?)?;
                let row_id = Uuid::from_str(&value)?;
                Ok($name(row_id))
            }
        }
    }
}

macro_rules! generate_redis_convert {
    ($name: ident) => {
        #[cfg(feature = "stateless")]
        impl redis::FromRedisValue for $name {
            fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
                let result: redis::RedisResult<i32> = redis::FromRedisValue::from_redis_value(v);
                match result {
                    Ok(value) => Ok(FromPrimitive::from_i32(value).unwrap_or_default()),
                    Err(_) => Ok($name::default())
                }
            }
        }
    }
}

/* **************************************************************************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct SendMessage {
    pub user_id: Arc<UserId>,
    pub message: String
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct UserAuthenticated(pub UserJwt);

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct WebsocketMessage(pub String);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserMetaInformation {
    pub id: UserMetaId,
    pub name: String,
    pub meta: MetaType<UserMetaAccess>
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserInformationModel {
    pub id: UserId,
    pub name: Option<String>,
    pub email: Option<String>,
    pub device_id: Option<String>,
    pub custom_id: Option<String>,
    pub metas: Option<UserMetaCollection>,
    pub user_type: UserType,
    pub online: bool,
    pub insert_date: i32,
    pub last_login_date: i32,
}

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* **************************************************************************************************************** */
#[derive(Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr, Copy, Clone, Default)]
#[repr(u8)]
pub enum UserType {
    #[default]
    User = 1,
    Mod = 2,
    Admin = 3
}

#[derive(Default, Clone, Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum CreateRoomAccessType {
    #[default]
    Public = 0,
    Private = 1,
    Friend = 2
}

#[derive(Default, Debug, Eq, PartialEq, PartialOrd, Clone, Serialize_repr, Deserialize_repr, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum RoomUserType {
    #[default]
    User = 1,
    Moderator = 2,
    Owner = 3,
}

/* **************************************************************************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* **************************************************************************************************************** */
impl WebsocketMessage {
    pub fn success<T: Debug + Serialize + DeserializeOwned>(request_id: Option<usize>, message: T) -> WebsocketMessage {
        let message = serde_json::to_string(&GenericAnswer::success(request_id, message));
        WebsocketMessage(message.unwrap())
    }
    
    pub fn fail<T: Debug + Serialize + DeserializeOwned>(request_id: Option<usize>, message: T) -> WebsocketMessage {
        let message = serde_json::to_string(&GenericAnswer::fail(request_id, message));
        WebsocketMessage(message.unwrap())
    }
}

/* **************************************************************************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* **************************************************************************************************************** */
impl Copy for RoomId { }

impl From<UserType> for i32 {
    fn from(user_type: UserType) -> Self {
        match user_type {
            UserType::User => 1,
            UserType::Mod => 2,
            UserType::Admin => 3,
        }
    }
}

impl From<i32> for UserType {
    fn from(user_type: i32) -> Self {
        match user_type {
            1 => UserType::User,
            2 => UserType::Mod,
            3 => UserType::Admin,
            _ => UserType::default()
        }
    }
}

impl From<CreateRoomAccessType> for i32 {
    fn from(item: CreateRoomAccessType) -> Self {
        match item {
            CreateRoomAccessType::Public => 0,
            CreateRoomAccessType::Private => 1,
            CreateRoomAccessType::Friend => 2,
        }
    }
}

impl From<i32> for CreateRoomAccessType {
    fn from(user_type: i32) -> Self {
        match user_type {
            0 => CreateRoomAccessType::Public,
            1 => CreateRoomAccessType::Private,
            2 => CreateRoomAccessType::Friend,
            _ => CreateRoomAccessType::default()
        }
    }
}

impl From<RoomUserType> for i32 {
    fn from(item: RoomUserType) -> Self {
        match item {
            RoomUserType::User => 1,
            RoomUserType::Moderator => 2,
            RoomUserType::Owner => 3,
        }
    }
}

impl From<i32> for RoomUserType {
    fn from(user_type: i32) -> Self {
        match user_type {
            1 => RoomUserType::User,
            2 => RoomUserType::Moderator,
            3 => RoomUserType::Owner,
            _ => RoomUserType::default()
        }
    }
}

/* **************************************************************************************************************** */
/* ************************************************* MACROS CALL ************************************************** */
/* **************************************************************************************************************** */
generate_type!(UserId);
generate_type!(UserMetaId);
generate_type!(SessionId);
generate_type!(RoomId);
generate_type!(RoomMetaId);
generate_type!(RoomTagId);
generate_type!(RoomUserId);
generate_type!(RoomUserBanId);
generate_type!(RoomUserRequestId);

generate_redis_convert!(CreateRoomAccessType);
generate_redis_convert!(RoomUserType);

/* **************************************************************************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */
