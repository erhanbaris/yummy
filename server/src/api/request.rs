use std::collections::HashMap;

use general::{model::{UserId, UserType, CreateRoomAccessType, RoomId, RoomUserType}, meta::{MetaType, RoomMetaAccess, UserMetaAccess, MetaAction}, password::Password};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "auth_type")]
pub enum RequestAuthType {
    Email {
        email: String,
        password: Password,

        #[serde(default, rename = "create")]
        if_not_exist_create: bool
    },
    DeviceId {
        id: String
    },
    CustomId {
        id: String
    },
    Refresh {
        token: String
    },
    Restore {
        token: String
    },
    Logout
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "user_type")]
pub enum RequestUserType {
    Me,
    Get {
        user: UserId
    },
    Update {
        name: Option<String>,
        email: Option<String>,
        password: Option<String>,
        device_id: Option<String>,
        custom_id: Option<String>,

        #[serde(rename = "type")]
        user_type: Option<UserType>,

        meta: Option<HashMap<String, MetaType<UserMetaAccess>>>,
        meta_action: Option<MetaAction>
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "room_type")]
pub enum RequestRoomType {
    Create {
        #[serde(default)]
        name: Option<String>,

        #[serde(default)]
        description: Option<String>,

        #[serde(default)]
        access_type: CreateRoomAccessType,

        #[serde(default)]
        join_request: bool,

        #[serde(default)]
        max_user: usize,

        #[serde(default)]
        tags: Vec<String>,

        #[serde(default)]
        metas: Option<HashMap<String, MetaType<RoomMetaAccess>>>
    },
    Join {
        room: RoomId,

        #[serde(default)]
        room_user_type: RoomUserType,
    },
    Disconnect {
        room: RoomId
    },
    Message {
        room: RoomId,
        message: String,
    },
    Update {
        room: RoomId,

        #[serde(default)]
        name: Option<String>,

        #[serde(default)]
        description: Option<String>,

        #[serde(default)]
        metas: Option<HashMap<String, MetaType<RoomMetaAccess>>>,
        
        #[serde(default)]
        meta_action: Option<MetaAction>,

        #[serde(default)]
        access_type: Option<CreateRoomAccessType>,

        #[serde(default)]
        join_request: Option<bool>,
        
        #[serde(default)]
        max_user: Option<usize>,
        
        #[serde(default)]
        tags: Option<Vec<String>>,
        
        #[serde(default)]
        user_permission: Option<HashMap<UserId, RoomUserType>>
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum Request {
    Auth {
        #[serde(flatten)]
        auth_type: RequestAuthType
    },
    User {
        #[serde(flatten)]
        user_type: RequestUserType
    },
    Room {
        #[serde(flatten)]
        room_type: RequestRoomType
    }
}