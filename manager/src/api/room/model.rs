use std::{fmt::Debug, sync::Arc};

use actix::prelude::Message;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use validator::Validate;

use general::{auth::UserAuth, model::{CreateRoomAccessType, RoomId, RoomUserType, UserId}, client::ClientTrait};


#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct CreateRoomRequest {
    pub user: Arc<Option<UserAuth>>,
    pub disconnect_from_other_room: bool,
    pub name: Option<String>,
    pub access_type: CreateRoomAccessType,
    pub max_user: usize,
    pub tags: Vec<String>,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct JoinToRoomRequest {
    pub user: Arc<Option<UserAuth>>,
    pub room: RoomId,
    pub room_user_type: RoomUserType,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct DisconnectFromRoomRequest {
    pub user: Arc<Option<UserAuth>>,
    pub room: RoomId,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct MessageToRoomRequest {
    pub user: Arc<Option<UserAuth>>,
    pub room: RoomId,
    pub message: String,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Error, Debug)]
pub enum RoomError {
    #[error("User joined to other room")]
    UserJoinedOtherRoom,

    #[error("Room not found")]
    RoomNotFound
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum RoomResponse {
    UserJoinedToRoom {
        user: UserId,
        room: RoomId
    },
    UserDisconnectedFromRoom {
        user: UserId,
        room: RoomId
    },
    MessageFromRoom {
        user: UserId,
        room: RoomId,
        message: Arc<String>
    }
}

impl From<RoomResponse> for String {
    fn from(source: RoomResponse) -> Self {
        serde_json::to_string(&source).unwrap_or_default()
    }
}

impl From<String> for RoomResponse {
    fn from(source: String) -> Self {
        serde_json::from_str(&source).unwrap()
    }
}
