use std::collections::HashMap;

use general::{model::{UserId, UserType, CreateRoomAccessType, RoomId, RoomUserType}, meta::MetaType, password::Password};
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

        meta: Option<HashMap<String, MetaType>>
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "room_type")]
pub enum RequestRoomType {
    Create {
        disconnect_from_other_room: bool,
        name: Option<String>,
        access_type: CreateRoomAccessType,
        max_user: usize,
        tags: Vec<String>
    },
    Join {
        room: RoomId,
        room_user_type: RoomUserType,
    },
    Disconnect {
        room: RoomId
    },
    Message {
        room: RoomId,
        message: String,
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