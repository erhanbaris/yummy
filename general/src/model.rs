use std::{fmt::Debug, cell::RefCell};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use actix::MessageResponse;
use uuid::Uuid;

use actix::prelude::Message;

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

#[derive(Message)]
#[rtype(result = "()")]
pub struct WebsocketMessage(pub String);

#[derive(Default, Debug)]
pub struct YummyState {
    user_sessions: Map<UserId, RefCell<Vec<SessionId>>>,
    session_user_id: Map<SessionId, UserId>
}

impl YummyState {
    pub fn is_user_online(&self, user_id: UserId) -> bool {
        self.user_sessions.get(&user_id).is_some()
    }

    pub fn new_session(&self, user_id: UserId) -> SessionId {
        let session_id = SessionId::new();
        self.session_user_id.insert(session_id.clone(), user_id);
        
        match self.user_sessions.get(&user_id) {
            Some(sessions) => sessions.1.borrow_mut().push(session_id.clone()),
            None => {
                self.user_sessions.insert(user_id.clone(), RefCell::new(vec![session_id.clone()]));
            }
        };

        session_id
    }
}