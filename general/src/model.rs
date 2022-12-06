use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::{fmt::Debug, borrow::Borrow};


use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_repr::{Serialize_repr, Deserialize_repr};

use actix::MessageResponse;
use actix::prelude::Message;

use uuid::Uuid;

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

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct WebsocketMessage(pub String);

#[derive(Default, Debug)]
pub struct UserState {
    pub user_id: UserId,
    pub session: SessionId,
    pub room: Cell<Option<RoomId>>
}

#[derive(Default, Debug)]
pub struct RoomState {
    max_user: usize,
    pub room_id: RoomId,
    pub users: Mutex<HashSet<UserId>>
}

#[derive(Default, Debug)]
pub struct YummyState {
    user: Mutex<HashMap<UserId, UserState>>,
    room: Mutex<HashMap<RoomId, RoomState>>,
    session_to_user: Mutex<HashMap<SessionId, UserId>>,
}

impl YummyState {
    #[tracing::instrument(name="is_user_online", skip(self))]
    pub fn is_user_online<T: Borrow<UserId> + std::fmt::Debug>(&self, user_id: T) -> bool {
        self.user.lock().contains_key(user_id.borrow())
    }

    #[tracing::instrument(name="is_session_online", skip(self))]
    pub fn is_session_online<T: Borrow<SessionId> + std::fmt::Debug>(&self, session_id: T) -> bool {
        self.session_to_user.lock().contains_key(session_id.borrow())
    }

    #[tracing::instrument(name="new_session", skip(self))]
    pub fn new_session(&self, user_id: UserId) -> SessionId {
        let session_id = SessionId::new();
        self.session_to_user.lock().insert(session_id.clone(), user_id);
        self.user.lock().insert(user_id.clone(), UserState { user_id: user_id, session: session_id.clone(), room: Cell::new(None) });
        session_id
    }

    #[tracing::instrument(name="close_session", skip(self))]
    pub fn close_session<T: Borrow<SessionId> + std::fmt::Debug>(&self, session_id: T) -> Option<UserState> {
        let removed = self.session_to_user.lock().remove(session_id.borrow());

        match removed {
            Some(removed) => self.user.lock().remove(&removed),
            None => None
        }
    }

    #[tracing::instrument(name="get_user_room", skip(self))]
    pub fn get_user_room<T: Borrow<UserId> + std::fmt::Debug>(&self, user_id: T) -> Option<RoomId> {
        self.user.lock().get(user_id.borrow()).and_then(|user| user.room.get())
    }

    #[tracing::instrument(name="set_user_room", skip(self))]
    pub fn set_user_room<T: Borrow<UserId> + std::fmt::Debug>(&self, user_id: T, room_id: RoomId){
        if let Some(user) = self.user.lock().get(user_id.borrow()) {
            user.room.set(Some(room_id));
        }
    }
}

#[derive(Debug)]
pub enum CreateRoomAccessType {
    Public,
    Private,
    Friend,
    Tag(String)
}

#[derive(Debug, Default)]
#[repr(u8)]
pub enum RoomUserType {
    #[default]
    User = 1,
    Owner = 2
}

#[cfg(test)]
mod tests {
    use crate::model::*;
    use anyhow::Ok;

    #[test]
    fn state_1() -> anyhow::Result<()> {
        let state = YummyState::default();
        let user_id = UserId::new();
        let session_id = state.new_session(user_id);

        assert!(state.is_session_online(session_id.clone()));
        assert!(state.is_user_online(user_id.clone()));

        state.close_session(session_id.clone());

        assert!(!state.is_session_online(session_id.clone()));
        assert!(!state.is_user_online(user_id.clone()));

        Ok(())
    }
    #[test]
    fn state_2() -> anyhow::Result<()> {
        let state = YummyState::default();

        state.close_session(SessionId::new());

        assert!(!state.is_session_online(SessionId::new()));
        assert!(!state.is_user_online(UserId::new()));

        Ok(())
    }
}
