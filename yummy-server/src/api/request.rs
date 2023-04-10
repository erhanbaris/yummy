use std::collections::HashMap;

use serde_json::Value;
use yummy_cache::state::RoomInfoTypeVariant;
use yummy_general::password::Password;
use yummy_model::{UserId, UserType, CreateRoomAccessType, RoomId, RoomUserType, meta::{MetaType, RoomMetaAccess, UserMetaAccess, MetaAction}};
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
        user_id: UserId
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

        metas: Option<HashMap<String, MetaType<UserMetaAccess>>>,

        #[serde(default)]
        meta_action: MetaAction
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
        room_id: RoomId,

        #[serde(default)]
        room_user_type: RoomUserType,
    },
    
    #[serde(rename = "RoomDisconnect")]
    Disconnect {
        room_id: RoomId
    },
    
    #[serde(rename = "MessageToRoom")]
    Message {
        room_id: RoomId,
        message: Value,
    },
    
    #[serde(rename = "KickUserFromroom")]
    Kick {
        room_id: RoomId,
        user_id: UserId,
    },
    
    #[serde(rename = "BanUserFromroom")]
    Ban {
        room_id: RoomId,
        user_id: UserId,
    },

    #[serde(rename = "RoomList")]
    List {
        #[serde(default)]
        tag: Option<String>,

        #[serde(default)]
        members: Vec<RoomInfoTypeVariant>,
    },
    
    #[serde(rename = "UpdateRoom")]
    Update {
        room_id: RoomId,

        #[serde(default)]
        name: Option<String>,

        #[serde(default)]
        description: Option<String>,

        #[serde(default)]
        metas: Option<HashMap<String, MetaType<RoomMetaAccess>>>,
        
        #[serde(default)]
        meta_action: MetaAction,

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
        #[serde(default)]
        request_id: Option<usize>,

        #[serde(flatten)]
        auth_type: RequestAuthType
    },
    
    User {
        #[serde(default)]
        request_id: Option<usize>,

        #[serde(flatten)]
        user_type: RequestUserType
    },
    
    Room {
        #[serde(default)]
        request_id: Option<usize>,

        #[serde(flatten)]
        room_type: RequestRoomType
    }
}