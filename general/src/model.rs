use std::{fmt::Debug, borrow::Borrow};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_repr::{Serialize_repr, Deserialize_repr};

use actix::MessageResponse;
use actix::prelude::Message;

use uuid::Uuid;

use lockfree::prelude::Map;

#[derive(Default, MessageResponse, Deserialize, Serialize, Eq, PartialEq, Debug, Copy, Clone, Hash, Ord, PartialOrd)]
pub struct UserId(pub Uuid);

#[derive(Default, MessageResponse, Deserialize, Serialize, Eq, PartialEq, Debug, Clone, Hash, Ord, PartialOrd)]
pub struct SessionId(pub Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
    
    pub fn empty(&self) -> bool { self.0.is_nil() }

    pub fn get(&self) -> Uuid {
        self.0
    }
}

#[derive(Default, MessageResponse, Deserialize, Serialize, Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct RoomId(pub Uuid);

impl FromStr for RoomId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(RoomId)
    }
}

unsafe impl Send for RoomId {}
unsafe impl Sync for RoomId {}

unsafe impl Send for UserId {}
unsafe impl Sync for UserId {}

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

unsafe impl Send for UserType {}
unsafe impl Sync for UserType {}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct WebsocketMessage(pub String);

#[derive(Default, Debug)]
pub struct YummyState {
    user_to_session: Map<UserId, SessionId>,
    session_to_user: Map<SessionId, UserId>
}

impl YummyState {
    #[tracing::instrument(name="is_user_online", skip(self))]
    pub fn is_user_online<T: Borrow<UserId> + std::fmt::Debug>(&self, user_id: T) -> bool {
        self.user_to_session.get(user_id.borrow()).is_some()
    }

    #[tracing::instrument(name="is_session_online", skip(self))]
    pub fn is_session_online<T: Borrow<SessionId> + std::fmt::Debug>(&self, session_id: T) -> bool {
        self.session_to_user.get(session_id.borrow()).is_some()
    }

    #[tracing::instrument(name="new_session", skip(self))]
    pub fn new_session(&self, user_id: UserId) -> SessionId {
        let session_id = SessionId::new();
        self.session_to_user.insert(session_id.clone(), user_id);
        self.user_to_session.insert(user_id, session_id.clone());
        session_id
    }

    #[tracing::instrument(name="close_session", skip(self))]
    pub fn close_session<T: Borrow<SessionId> + std::fmt::Debug>(&self, session_id: T) -> Option<UserId> {
        let removed = self.session_to_user.remove(session_id.borrow());

        match removed {
            Some(removed) => self.user_to_session.remove(removed.val()).map(|item| item.0),
            None => None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::*;
    use anyhow::Ok;

    #[test]
    fn state_1() -> anyhow::Result<()> {
        let state = YummyState::default();
        let user_id = UserId(Uuid::new_v4());
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

        state.close_session(SessionId(Uuid::new_v4()));

        assert!(!state.is_session_online(SessionId(Uuid::new_v4())));
        assert!(!state.is_user_online(UserId(Uuid::new_v4())));

        Ok(())
    }
}
