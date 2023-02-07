use std::{fmt::Debug, sync::Arc, collections::HashMap, borrow::Cow};

use actix::prelude::Message;
use serde::Serialize;
use thiserror::Error;
use validator::Validate;

use general::{auth::UserAuth, model::{CreateRoomAccessType, RoomId, RoomUserType, UserId}, client::ClientTrait, state::{RoomUserInformation, RoomInfoTypeVariant, RoomInfoTypeCollection}, meta::{MetaType, RoomMetaAccess, MetaAction}};


#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct CreateRoomRequest {
    pub auth: Arc<Option<UserAuth>>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub access_type: CreateRoomAccessType,
    pub join_request: bool,
    pub max_user: usize,
    pub tags: Vec<String>,
    pub metas: Option<HashMap<String, MetaType<RoomMetaAccess>>>,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct JoinToRoomRequest {
    pub auth: Arc<Option<UserAuth>>,
    pub room: RoomId,
    pub room_user_type: RoomUserType,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct WaitingRoomJoins {
    pub auth: Arc<Option<UserAuth>>,
    pub room: RoomId,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct KickUserFromRoom {
    pub auth: Arc<Option<UserAuth>>,
    pub room: RoomId,
    pub user: UserId,
    pub ban: bool,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct ProcessWaitingUser {
    pub auth: Arc<Option<UserAuth>>,
    pub room: RoomId,
    pub user: UserId,
    pub status: bool,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug, Clone)]
#[rtype(result = "()")]
pub struct DisconnectFromRoomRequest {
    pub auth: Arc<Option<UserAuth>>,
    pub room: RoomId,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct MessageToRoomRequest {
    pub auth: Arc<Option<UserAuth>>,
    pub room: RoomId,
    pub message: String,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct RoomListRequest {
    pub tag: Option<String>,
    pub members: Vec<RoomInfoTypeVariant>,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct GetRoomRequest {
    pub auth: Arc<Option<UserAuth>>,
    pub room: RoomId,
    pub members: Vec<RoomInfoTypeVariant>,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct UpdateRoom {
    pub auth: Arc<Option<UserAuth>>,
    pub room_id: RoomId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub join_request: Option<bool>,
    pub socket: Arc<dyn ClientTrait + Sync + Send>,
    pub metas: Option<HashMap<String, MetaType<RoomMetaAccess>>>,
    pub meta_action: Option<MetaAction>,
    pub access_type: Option<CreateRoomAccessType>,
    pub max_user: Option<usize>,
    pub tags: Option<Vec<String>>,
    pub user_permission: Option<HashMap<UserId, RoomUserType>>
}

#[derive(Error, Debug)]
pub enum RoomError {
    #[error("User joined to other room")]
    UserJoinedOtherRoom,

    #[error("Room not found")]
    RoomNotFound,

    #[error("Update information missing")]
    UpdateInformationMissing,

    #[error("Meta limit over to maximum")]
    MetaLimitOverToMaximum,

    #[error("User does not have enough permission")]
    UserDoesNotHaveEnoughPermission,

    #[error("User is not in the room")]
    UserNotInTheRoom,

    #[error("Banned from room")]
    BannedFromRoom
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum RoomResponse<'a> {
    RoomCreated { room: RoomId },
    Joined {
        room: &'a RoomId,
        room_name: Cow<'a, Option<String>>,
        users: Cow<'a, Vec<RoomUserInformation>>,
        metas: Cow<'a, HashMap<String, MetaType<RoomMetaAccess>>>
    },
    JoinRequested {
        room: &'a RoomId,
    },
    JoinRequestDeclined {
        room: &'a RoomId,
    },
    WaitingRoomJoins {
        room: &'a RoomId,
        users: HashMap<Arc<UserId>, RoomUserType>,
    },
    NewJoinRequest {
        room: &'a RoomId,
        user: &'a UserId,
        user_type: RoomUserType
    },
    UserJoinedToRoom {
        user: &'a UserId,
        room: &'a RoomId
    },
    UserDisconnectedFromRoom {
        user: &'a UserId,
        room: &'a RoomId
    },
    DisconnectedFromRoom {
        room: &'a RoomId
    },
    MessageFromRoom {
        user: &'a UserId,
        room: &'a RoomId,
        message: &'a String
    },
    RoomList {
        rooms: Vec<RoomInfoTypeCollection>
    },
    RoomInfo {
        #[serde(flatten)]
        room: RoomInfoTypeCollection
    }
}

impl<'a> From<RoomResponse<'a>> for String {
    fn from(source: RoomResponse) -> Self {
        serde_json::to_string(&source).unwrap()
    }
}
