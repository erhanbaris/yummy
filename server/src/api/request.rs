use std::collections::HashMap;

use general::{model::{UserId, UserType}, meta::MetaType};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "auth_type")]
pub enum RequestAuthType {
    Email {
        email: String,
        password: String,

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
    Create { }
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