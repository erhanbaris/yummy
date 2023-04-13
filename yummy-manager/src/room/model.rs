use std::{fmt::Debug, sync::Arc, collections::HashMap, borrow::Cow};

use actix::prelude::Message;
use serde_json::Value;
use yummy_cache::state::{RoomInfoTypeVariant, RoomUserInformation, RoomInfoTypeCollection};
use yummy_general::client::ClientTrait;
use yummy_model::{auth::UserAuth, CreateRoomAccessType, meta::{RoomMetaAccess, MetaType, MetaAction}, RoomId, RoomUserType, UserId};
use serde::Serialize;
use thiserror::Error;
use validator::Validate;


#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct CreateRoomRequest {
    pub request_id: Option<usize>, 
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
    pub request_id: Option<usize>, 
    pub auth: Arc<Option<UserAuth>>,
    pub room_id: RoomId,
    pub room_user_type: RoomUserType,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct WaitingRoomJoins {
    pub request_id: Option<usize>, 
    pub auth: Arc<Option<UserAuth>>,
    pub room_id: RoomId,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct KickUserFromRoom {
    pub request_id: Option<usize>, 
    pub auth: Arc<Option<UserAuth>>,
    pub room_id: RoomId,
    pub user_id: UserId,
    pub ban: bool,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct ProcessWaitingUser {
    pub request_id: Option<usize>, 
    pub auth: Arc<Option<UserAuth>>,
    pub room_id: RoomId,
    pub user_id: UserId,
    pub status: bool,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug, Clone)]
#[rtype(result = "()")]
pub struct DisconnectFromRoomRequest {
    pub request_id: Option<usize>, 
    pub auth: Arc<Option<UserAuth>>,
    pub room_id: RoomId,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct MessageToRoomRequest {
    pub request_id: Option<usize>, 
    pub auth: Arc<Option<UserAuth>>,
    pub room_id: RoomId,
    pub message: Value,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct RoomListRequest {
    pub request_id: Option<usize>, 
    pub tag: Option<String>,
    pub members: Vec<RoomInfoTypeVariant>,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct GetRoomRequest {
    pub request_id: Option<usize>, 
    pub auth: Arc<Option<UserAuth>>,
    pub room_id: RoomId,
    pub members: Vec<RoomInfoTypeVariant>,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct UpdateRoom {
    pub request_id: Option<usize>, 
    pub auth: Arc<Option<UserAuth>>,
    pub room_id: RoomId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub join_request: Option<bool>,
    pub socket: Arc<dyn ClientTrait + Sync + Send>,
    pub metas: Option<HashMap<String, MetaType<RoomMetaAccess>>>,
    pub meta_action: MetaAction,
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
    RoomCreated { room_id: RoomId },
    Joined {
        room_id: &'a RoomId,
        room_name: Cow<'a, Option<String>>,
        users: Cow<'a, Vec<RoomUserInformation>>,
        metas: Cow<'a, HashMap<String, MetaType<RoomMetaAccess>>>
    },
    JoinRequested {
        room_id: &'a RoomId,
    },
    JoinRequestDeclined {
        room_id: &'a RoomId,
    },
    WaitingRoomJoins {
        room_id: &'a RoomId,
        users: HashMap<Arc<UserId>, RoomUserType>,
    },
    NewJoinRequest {
        room_id: &'a RoomId,
        user_id: &'a UserId,
        user_type: RoomUserType
    },
    UserJoinedToRoom {
        user_id: &'a UserId,
        room_id: &'a RoomId
    },
    UserDisconnectedFromRoom {
        user_id: &'a UserId,
        room_id: &'a RoomId
    },
    DisconnectedFromRoom {
        room_id: &'a RoomId
    },
    MessageFromRoom {
        #[serde(skip_serializing_if = "Option::is_none")]
        user_id: Option<&'a UserId>,
        room_id: &'a RoomId,
        message: &'a Value
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
