use std::fmt::Debug;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use actix::MessageResponse;

#[derive(Default, MessageResponse, Deserialize, Serialize, Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct UserId(pub u64);

#[derive(Default, MessageResponse, Deserialize, Serialize, Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct ConnectionId(pub usize);

impl PartialEq<usize> for ConnectionId {
    fn eq(&self, other: &usize) -> bool { self.0 == *other }
}

impl ConnectionId {
    pub fn empty(&self) -> bool { self.0 == 0 }
}

#[derive(Default, MessageResponse, Deserialize, Serialize, Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct RoomId(pub usize);

impl PartialEq<usize> for RoomId {
    fn eq(&self, other: &usize) -> bool { self.0 == *other }
}

impl FromStr for RoomId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(RoomId(s.parse::<usize>().unwrap_or_default()))
    }
}

unsafe impl Send for RoomId {}
unsafe impl Sync for RoomId {}

unsafe impl Send for UserId {}
unsafe impl Sync for UserId {}

unsafe impl Send for ConnectionId {}
unsafe impl Sync for ConnectionId {}
