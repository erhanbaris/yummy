use std::collections::HashMap;

use general::{model::{UserId, UserType, CreateRoomAccessType, RoomId, RoomUserType}, meta::{MetaType, RoomMetaAccess, UserMetaAccess, MetaAction}, password::Password};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum RequestAuthType {
    #[serde(rename = "AuthEmail")]
    Email {
        email: String,
        password: Password,

        #[serde(default, rename = "create")]
        if_not_exist_create: bool
    },

    #[serde(rename = "AuthDeviceId")]
    DeviceId {
        id: String
    },
    
    #[serde(rename = "AuthCustomId")]
    CustomId {
        id: String
    },
    
    #[serde(rename = "RefreshToken")]
    Refresh {
        token: String
    },
    
    #[serde(rename = "RestoreToken")]
    Restore {
        token: String
    },
    
    #[serde(rename = "Logout")]
    Logout
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum RequestUserType {
    #[serde(rename = "Me")]
    Me,

    #[serde(rename = "GetUser")]
    Get {
        user: UserId
    },

    #[serde(rename = "UpdateUser")]
    Update {
        name: Option<String>,
        email: Option<String>,
        password: Option<String>,
        device_id: Option<String>,
        custom_id: Option<String>,

        #[serde(rename = "user_type")]
        user_type: Option<UserType>,

        meta: Option<HashMap<String, MetaType<UserMetaAccess>>>,
        meta_action: Option<MetaAction>
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum RequestRoomType {
    #[serde(rename = "CreateRoom")]
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
    
    #[serde(rename = "JoinToRoom")]
    Join {
        room: RoomId,

        #[serde(default)]
        room_user_type: RoomUserType,
    },
    
    #[serde(rename = "RoomDisconnect")]
    Disconnect {
        room: RoomId
    },
    
    #[serde(rename = "MessageToRoom")]
    Message {
        room: RoomId,
        message: String,
    },
    
    #[serde(rename = "UpdateRoom")]
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
#[serde(untagged)]
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