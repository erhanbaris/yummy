use std::fmt::Debug;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use actix::MessageResponse;
use uuid::Uuid;

#[derive(Default, MessageResponse, Deserialize, Serialize, Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct UserId(pub Uuid);

#[derive(Default, MessageResponse, Deserialize, Serialize, Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct SessionId(pub Uuid);

impl SessionId {
    pub fn empty(&self) -> bool { self.0.is_nil() }
}

#[derive(Default, MessageResponse, Deserialize, Serialize, Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct RoomId(pub Uuid);

impl FromStr for RoomId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(|id| RoomId(id))
    }
}

unsafe impl Send for RoomId {}
unsafe impl Sync for RoomId {}

unsafe impl Send for UserId {}
unsafe impl Sync for UserId {}

unsafe impl Send for SessionId {}
unsafe impl Sync for SessionId {}
