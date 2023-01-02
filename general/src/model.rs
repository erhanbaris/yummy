use std::collections::HashMap;
use std::hash::Hash;
use std::fmt::Debug;
use std::str::FromStr;

use serde::de::DeserializeOwned;
use parking_lot::Mutex;
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

use uuid::Uuid;

use crate::auth::UserJwt;
use crate::web::GenericAnswer;

macro_rules! generate_type {
    ($name:ident) => {
        
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
        }

        impl Default for $name {
            fn default() -> Self {
                Self(uuid::Uuid::new_v4())
            }
        }
        impl AsRef<$name> for $name {
            fn as_ref(&self) -> &$name {
                self
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

        impl From<&str> for $name {
            fn from(data: &str) -> Self {
                $name(uuid::Uuid::parse_str(data).unwrap_or_default())
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

generate_type!(UserId);
generate_type!(UserMetaId);
generate_type!(SessionId);
generate_type!(RoomId);
generate_type!(RoomMetaId);
generate_type!(RoomTagId);
generate_type!(RoomUserId);

impl Copy for RoomId { }

#[derive(Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr, Copy, Clone, Default)]
#[repr(u8)]
pub enum UserType {
    #[default]
    User = 1,
    Mod = 2,
    Admin = 3
}

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

#[cfg_attr(feature = "stateless", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct UserState {
    pub user_id: UserId,
    pub session: SessionId,
    pub name: Option<String>,
    pub user_type: UserType,

    #[cfg(not(feature = "stateless"))]
    pub room: std::cell::Cell<Option<RoomId>>
}

#[derive(Default, Debug)]
pub struct RoomState {
    pub room_id: RoomId,
    pub name: Option<String>,
    pub access_type: CreateRoomAccessType,
    pub max_user: usize,
    pub tags: Vec<String>,
    pub insert_date: i32,
    pub users: Mutex<HashMap<UserId, RoomUserType>>
}

#[derive(Default, Clone, Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq)]
#[repr(u8)]
pub enum CreateRoomAccessType {
    #[default]
    Public = 0,
    Private = 1,
    Friend = 2
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

#[derive(Default, Debug, Eq, PartialEq, PartialOrd, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum RoomUserType {
    #[default]
    User = 1,
    Owner = 2,
    Moderator = 3
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct UserAuthenticated(pub UserJwt);

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct WebsocketMessage(pub String);

impl WebsocketMessage {
    pub fn success<T: Debug + Serialize + DeserializeOwned>(message: T) -> WebsocketMessage {
        let message = serde_json::to_string(&GenericAnswer::success(message));
        WebsocketMessage(message.unwrap())
    }
    
    pub fn fail<T: Debug + Serialize + DeserializeOwned>(message: T) -> WebsocketMessage {
        let message = serde_json::to_string(&GenericAnswer::fail(message));
        WebsocketMessage(message.unwrap())
    }
}


#[cfg(test)]
mod tests {
    use crate::model::UserId;

    #[test]
    fn user_id() -> anyhow::Result<()> {
        let row_id = UserId::default();
        assert!(!row_id.get().is_nil());

        let row_id = UserId(uuid::Uuid::new_v4());
        assert!(!row_id.get().is_nil());

        let uuid_data = "85fc32fe-eaa5-46c3-b8e8-60bb658b5de7";
        let row_id: UserId = uuid_data.to_string().into();

        let new_uuid_data: String = row_id.to_string();
        assert_eq!(&new_uuid_data, uuid_data);

        Ok(())
    }
}
