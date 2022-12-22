use std::collections::HashSet;
use std::hash::Hash;
use std::fmt::Debug;


use serde::de::DeserializeOwned;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_repr::{Serialize_repr, Deserialize_repr};

use actix::MessageResponse;
use actix::prelude::Message;

use uuid::Uuid;

use crate::auth::UserJwt;
use crate::web::GenericAnswer;

#[derive(Default, MessageResponse, Deserialize, Serialize, Eq, PartialEq, Debug, Copy, Clone, Hash, Ord, PartialOrd)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    pub fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
    
    pub fn get(&self) -> Uuid {
        self.0
    }
}

#[derive(Default, MessageResponse, Deserialize, Serialize, Eq, PartialEq, Debug, Clone, Hash, Ord, PartialOrd)]
pub struct SessionId(Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
    
    pub fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
    
    pub fn get(&self) -> Uuid {
        self.0
    }
}

#[derive(Default, MessageResponse, Deserialize, Serialize, Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct RoomId(Uuid);

impl RoomId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
    
    pub fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
    
    pub fn get(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr, Clone, Default)]
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

    #[cfg(not(feature = "stateless"))]
    pub room: std::cell::Cell<Option<RoomId>>
}

#[derive(Default, Debug)]
pub struct RoomState {
    pub max_user: usize,
    pub room_id: RoomId,
    pub users: Mutex<HashSet<RoomUserInfo>>
}

#[derive(Default, Debug, Eq)]
pub struct RoomUserInfo {
    pub user_id: UserId,
    pub room_user_type: RoomUserType
}

impl Hash for RoomUserInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.user_id.hash(state)
    }
}

impl PartialEq for RoomUserInfo {
    fn eq(&self, other: &Self) -> bool {
        self.user_id == other.user_id
    }
}

impl RoomUserInfo {
    pub fn new(user_id: UserId, room_user_type: RoomUserType) -> Self {
        Self { user_id, room_user_type }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CreateRoomAccessType {
    Public,
    Private,
    Friend,
    Tag(String)
}

#[derive(Default, Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
#[repr(u8)]
pub enum RoomUserType {
    #[default]
    User = 1,
    Owner = 2
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
        WebsocketMessage(message.unwrap_or_default())
    }
    
    pub fn fail<T: Debug + Serialize + DeserializeOwned>(message: T) -> WebsocketMessage {
        let message = serde_json::to_string(&GenericAnswer::fail(message));
        WebsocketMessage(message.unwrap_or_default())
    }
}
