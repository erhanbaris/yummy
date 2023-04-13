pub mod resource;

#[cfg(not(feature = "stateless"))]
pub mod inmemory;

#[cfg(feature = "stateless")]
pub mod stateless;

#[cfg(not(feature = "stateless"))]
pub use crate::state::inmemory::YummyState;

#[cfg(feature = "stateless")]
pub use crate::state::stateless::YummyState;

#[cfg(test)]
mod test;

use std::collections::{HashMap, HashSet};
use std::borrow::Cow;
use std::sync::Arc;
use std::fmt::Debug;

use serde::de::Visitor;
use yummy_model::meta::{RoomMetaAccess, MetaType};
use yummy_model::{UserId, RoomUserType, CreateRoomAccessType, RoomId};
use serde::ser::SerializeMap;
use strum_macros::EnumDiscriminants;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum YummyStateError {
    #[error("Room not found")]
    RoomNotFound,

    #[error("User not found")]
    UserNotFound,
    
    #[error("User already in room")]
    UserAlreadInRoom,
    
    #[error("Already requested")]
    AlreadyRequested,
    
    #[error("User could not found in the room")]
    UserCouldNotFoundInRoom,
    
    #[error("Room has max users")]
    RoomHasMaxUsers,
    
    #[error("Cache could not readed")]
    CacheCouldNotReaded,
    
    #[error("Cache error {0}")]
    CacheError(#[from] anyhow::Error)
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct RoomUserInformation {
    pub user_id: Arc<UserId>,
    pub name: Option<String>,

    #[serde(rename = "type")]
    pub user_type: RoomUserType
}

#[derive(Debug, Clone, EnumDiscriminants, PartialEq, Deserialize)]
#[strum_discriminants(name(RoomInfoTypeVariant))]
pub enum RoomInfoType {
    RoomName(Option<String>),
    Description(Option<String>),
    Users(Vec<RoomUserInformation>),
    MaxUser(usize),
    UserLength(usize),
    AccessType(CreateRoomAccessType),
    Tags(Vec<String>),
    Metas(HashMap<String, MetaType<RoomMetaAccess>>),
    InsertDate(i32),
    JoinRequest(bool),
    BannedUsers(HashSet<UserId>)
}

impl From<RoomInfoTypeVariant> for u32 {
    fn from(value: RoomInfoTypeVariant) -> Self {
        match value {
            RoomInfoTypeVariant::RoomName => 0,
            RoomInfoTypeVariant::Description => 1,
            RoomInfoTypeVariant::Users => 2,
            RoomInfoTypeVariant::MaxUser => 3,
            RoomInfoTypeVariant::UserLength => 4,
            RoomInfoTypeVariant::AccessType => 5,
            RoomInfoTypeVariant::Tags => 6,
            RoomInfoTypeVariant::Metas => 7,
            RoomInfoTypeVariant::InsertDate => 8,
            RoomInfoTypeVariant::JoinRequest => 9,
            RoomInfoTypeVariant::BannedUsers => 10,
        }
    }
}

impl From<u32> for RoomInfoTypeVariant {
    fn from(value: u32) -> Self {
        match value {
            0 => RoomInfoTypeVariant::RoomName,
            1 => RoomInfoTypeVariant::Description,
            2 => RoomInfoTypeVariant::Users,
            3 => RoomInfoTypeVariant::MaxUser,
            4 => RoomInfoTypeVariant::UserLength,
            5 => RoomInfoTypeVariant::AccessType,
            6 => RoomInfoTypeVariant::Tags,
            7 => RoomInfoTypeVariant::Metas,
            8 => RoomInfoTypeVariant::InsertDate,
            9 => RoomInfoTypeVariant::JoinRequest,
            10 => RoomInfoTypeVariant::BannedUsers,
            _ => RoomInfoTypeVariant::RoomName
        }
    }
}

impl Serialize for RoomInfoTypeVariant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(u32::from(*self) as i32)
    }
}

impl<'de> Deserialize<'de> for RoomInfoTypeVariant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IntegerVisitor;
        impl<'de> Visitor<'de> for IntegerVisitor {
            type Value = RoomInfoTypeVariant;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an u64")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(RoomInfoTypeVariant::from(value as u32))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(RoomInfoTypeVariant::from(value as u32))
            }
        }

        deserializer.deserialize_i32(IntegerVisitor)
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RoomInfoTypeCollection {
    pub room_id: Option<RoomId>,
    pub items: Vec<RoomInfoType>
}

macro_rules! generate_room_type_getter {
    ($name: ident, $variant: path, $response: ty) => {
        pub fn $name(&self) -> Cow<'_, $response> {        
            for item in self.items.iter() {
                match item {
                    $variant(value) => return Cow::Borrowed(value),
                    _ => ()
                };
            }
    
            Cow::Owned(<$response>::default())
        }
    }
}

impl RoomInfoTypeCollection {       
    generate_room_type_getter!(get_room_name, RoomInfoType::RoomName, Option<String>);
    generate_room_type_getter!(get_description, RoomInfoType::Description, Option<String>);
    generate_room_type_getter!(get_users, RoomInfoType::Users, Vec<RoomUserInformation>);
    generate_room_type_getter!(get_max_user, RoomInfoType::MaxUser, usize);
    generate_room_type_getter!(get_user_length, RoomInfoType::UserLength, usize);
    generate_room_type_getter!(get_access_type, RoomInfoType::AccessType, CreateRoomAccessType);
    generate_room_type_getter!(get_tags, RoomInfoType::Tags, Vec<String>);
    generate_room_type_getter!(get_metas, RoomInfoType::Metas, HashMap<String, MetaType<RoomMetaAccess>>);
    generate_room_type_getter!(get_insert_date, RoomInfoType::InsertDate, i32);
    generate_room_type_getter!(get_join_request, RoomInfoType::JoinRequest, bool);
    generate_room_type_getter!(get_banned_users, RoomInfoType::BannedUsers, HashSet<UserId>);

}

impl Serialize for RoomInfoTypeCollection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut items = serializer.serialize_map(Some(self.items.len()))?;
        if let Some(room_id) = self.room_id {
            items.serialize_entry("id", &room_id)?;
        }

        for entry in self.items.iter() {
            match entry {
                RoomInfoType::RoomName(name) => items.serialize_entry("name", name),
                RoomInfoType::Description(description) => items.serialize_entry("description", description),
                RoomInfoType::Users(users) => items.serialize_entry("users", users),
                RoomInfoType::MaxUser(max_user) => items.serialize_entry("max-user", max_user),
                RoomInfoType::UserLength(user_length) => items.serialize_entry("user-length", user_length),
                RoomInfoType::AccessType(access_type) => items.serialize_entry("access-type", access_type),
                RoomInfoType::Tags(tags) => items.serialize_entry("tags", tags),
                RoomInfoType::Metas(tags) => items.serialize_entry("metas", tags),
                RoomInfoType::BannedUsers(banned_users) => items.serialize_entry("banned-users", banned_users),
                RoomInfoType::InsertDate(insert_date) => items.serialize_entry("insert-date", insert_date),
                RoomInfoType::JoinRequest(join_request) => items.serialize_entry("join-request", join_request),
            }?;
        }
        
        items.end()
    }
}
