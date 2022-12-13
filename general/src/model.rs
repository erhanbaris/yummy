use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;
use std::{fmt::Debug, borrow::Borrow};

use serde::de::DeserializeOwned;
use thiserror::Error;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_repr::{Serialize_repr, Deserialize_repr};

use actix::MessageResponse;
use actix::prelude::Message;

use uuid::Uuid;

use crate::auth::UserJwt;
use crate::client::ClientTrait;
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

#[derive(Debug)]
pub struct UserState {
    pub user_id: UserId,
    pub session: SessionId,
    pub room: Cell<Option<RoomId>>,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Default, Debug)]
pub struct RoomState {
    pub max_user: usize,
    pub room_id: RoomId,
    pub users: Mutex<HashSet<RoomUserInfo>>
}

#[derive(Default, Debug, Eq, PartialEq)]
pub struct RoomUserInfo {
    pub user_id: UserId,
    pub room_user_type: RoomUserType
}

impl Hash for RoomUserInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.user_id.hash(state)
    }
}

impl RoomUserInfo {
    pub fn new(user_id: UserId, room_user_type: RoomUserType) -> Self {
        Self { user_id, room_user_type }
    }
}

#[derive(Default, Debug)]
pub struct YummyState {
    user: Mutex<HashMap<UserId, UserState>>,
    room: Mutex<HashMap<RoomId, RoomState>>,
    session_to_user: Mutex<HashMap<SessionId, UserId>>,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum YummyStateError {
    #[error("Room not found")]
    RoomNotFound,
    
    #[error("User already in room")]
    UserAlreadInRoom,
    
    #[error("Room has max users")]
    RoomHasMaxUsers
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
    pub fn new_session(&self, user_id: UserId, socket: Arc<dyn ClientTrait + Sync + Send>) -> SessionId {
        let session_id = SessionId::new();
        self.session_to_user.lock().insert(session_id.clone(), user_id);
        self.user.lock().insert(user_id.clone(), UserState { user_id: user_id, session: session_id.clone(), room: Cell::new(None), socket });
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

    #[tracing::instrument(name="get_user_socket", skip(self))]
    pub fn get_user_socket<T: Borrow<UserId> + std::fmt::Debug>(&self, user_id: T) -> Option<Arc<dyn ClientTrait + Sync + Send>> {
        self.user.lock().get(user_id.borrow()).map(|user| user.socket.clone())
    }

    #[tracing::instrument(name="set_user_room", skip(self))]
    pub fn set_user_room<T: Borrow<UserId> + std::fmt::Debug>(&self, user_id: T, room_id: RoomId){
        if let Some(user) = self.user.lock().get(user_id.borrow()) {
            user.room.set(Some(room_id));
        }
    }

    #[tracing::instrument(name="create_room", skip(self))]
    pub fn create_room(&self, room_id: RoomId, max_user: usize) {
        self.room.lock().insert(room_id.clone(), RoomState { max_user, room_id, users: Mutex::new(HashSet::new()) });
    }

    #[tracing::instrument(name="join_to_room", skip(self))]
    pub fn join_to_room(&self, room_id: RoomId, user_id: UserId, user_type: RoomUserType) -> Result<(), YummyStateError> {

        // Get room
        match self.room.lock().get_mut(room_id.borrow()) {
            Some(room) => {
                let mut users = room.users.lock();
                let users_len = users.len();

                // If the max_user 0 or lower than users count, add to room
                if room.max_user == 0 || room.max_user > users_len {

                    // User alread in the room
                    if !users.insert(RoomUserInfo::new(user_id, user_type)) {
                        return Err(YummyStateError::UserAlreadInRoom);
                    }
                    Ok(())
                } else {
                    Err(YummyStateError::RoomHasMaxUsers)
                }
            }
            None => Err(YummyStateError::RoomNotFound)
        }
    }

    #[tracing::instrument(name="get_users_from_room", skip(self))]
    pub fn get_users_from_room(&self, room_id: RoomId) -> Result<Vec<UserId>, YummyStateError> {
        match self.room.lock().get_mut(room_id.borrow()) {
            Some(room) => Ok(room.users.lock().iter().map(|item| item.user_id.clone()).collect::<Vec<_>>()),
            None => Err(YummyStateError::RoomNotFound)
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

#[derive(Default, Debug, Eq, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum RoomUserType {
    #[default]
    User = 1,
    Owner = 2
}

#[cfg(test)]
mod tests {
    use crate::{model::*, client::EmptyClient};
    use anyhow::Ok;

    #[actix::test]
    async fn state_1() -> anyhow::Result<()> {
        let state = YummyState::default();
        let user_id = UserId::new();
        let session_id = state.new_session(user_id, Arc::new(EmptyClient::default()));

        assert!(state.is_session_online(session_id.clone()));
        assert!(state.is_user_online(user_id.clone()));

        state.close_session(session_id.clone());

        assert!(!state.is_session_online(session_id.clone()));
        assert!(!state.is_user_online(user_id.clone()));

        Ok(())
    }

    #[actix::test]
    async fn state_2() -> anyhow::Result<()> {
        let state = YummyState::default();

        state.close_session(SessionId::new());

        assert!(!state.is_session_online(SessionId::new()));
        assert!(!state.is_user_online(UserId::new()));

        Ok(())
    }
    
    #[actix::test]
    async fn room_tests() -> anyhow::Result<()> {
        let state = YummyState::default();
        let room_1 = RoomId::new();
        state.create_room(room_1, 2);

        let user_1 = UserId::new();
        let user_2 = UserId::new();
        let user_3 = UserId::new();

        state.join_to_room(room_1.clone(), user_1.clone(), RoomUserType::Owner)?;
        assert_eq!(state.join_to_room(room_1.clone(), user_1.clone(), RoomUserType::Owner).err().unwrap(), YummyStateError::UserAlreadInRoom);

        state.join_to_room(room_1.clone(), user_2.clone(), RoomUserType::Owner)?;
        assert_eq!(state.join_to_room(room_1.clone(), user_3.clone(), RoomUserType::Owner).err().unwrap(), YummyStateError::RoomHasMaxUsers);
        assert_eq!(state.join_to_room(room_1.clone(), user_2.clone(), RoomUserType::Owner).err().unwrap(), YummyStateError::RoomHasMaxUsers);

        assert_eq!(state.join_to_room(RoomId::new(), UserId::new(), RoomUserType::Owner).err().unwrap(), YummyStateError::RoomNotFound);

        Ok(())
    }
    
    #[actix::test]
    async fn room_unlimited_users_tests() -> anyhow::Result<()> {
        let state = YummyState::default();
        let room = RoomId::new();
        state.create_room(room, 0);

        for _ in 0..100_000 {
            state.join_to_room(room, UserId::new(), RoomUserType::Owner)?
        }

        Ok(())
    }
}
