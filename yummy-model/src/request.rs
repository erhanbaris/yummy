use std::collections::HashMap;

use serde_json::Value;
use strum_macros::EnumDiscriminants;
use strum_macros::IntoStaticStr;
use crate::state::RoomInfoTypeVariant;

use crate::password::Password;
use crate::{UserId, UserType, CreateRoomAccessType, RoomId, RoomUserType, meta::{MetaType, RoomMetaAccess, UserMetaAccess, MetaAction}};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, EnumDiscriminants, Debug)]
#[strum_discriminants(name(RequestAuthTypeVariant), derive(Deserialize, Serialize, IntoStaticStr))]
#[serde(tag = "type")]
pub enum RequestAuthType {
    #[strum_discriminants(serde(rename = "AuthEmail"))]
    #[serde(rename = "AuthEmail")]
    Email {
        email: String,
        password: Password,

        #[serde(default, rename = "create")]
        if_not_exist_create: bool
    },

    #[strum_discriminants(serde(rename = "AuthDeviceId"))]
    #[serde(rename = "AuthDeviceId")]
    DeviceId {
        id: String
    },
    
    #[strum_discriminants(serde(rename = "AuthCustomId"))]
    #[serde(rename = "AuthCustomId")]
    CustomId {
        id: String
    },
    
    #[strum_discriminants(serde(rename = "RefreshToken"))]
    #[serde(rename = "RefreshToken")]
    Refresh {
        token: String
    },
    
    #[strum_discriminants(serde(rename = "RestoreToken"))]
    #[serde(rename = "RestoreToken")]
    Restore {
        token: String
    },
    
    #[strum_discriminants(serde(rename = "Logout"))]
    #[serde(rename = "Logout")]
    Logout
}

#[derive(Deserialize, Serialize, EnumDiscriminants, Debug)]
#[strum_discriminants(name(RequestUserTypeVariant), derive(Deserialize, Serialize, IntoStaticStr))]
#[serde(tag = "type")]
pub enum RequestUserType {
    #[strum_discriminants(serde(rename = "Me"))]
    #[serde(rename = "Me")]
    Me,

    #[strum_discriminants(serde(rename = "GetUser"))]
    #[serde(rename = "GetUser")]
    Get {
        user_id: UserId
    },

    #[strum_discriminants(serde(rename = "UpdateUser"))]
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

#[derive(Deserialize, Serialize, EnumDiscriminants, Debug)]
#[strum_discriminants(name(RequestRoomTypeVariant), derive(Deserialize, Serialize, IntoStaticStr))]
#[serde(tag = "type")]
pub enum RequestRoomType {
    #[strum_discriminants(serde(rename = "CreateRoom"))]
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
    
    #[strum_discriminants(serde(rename = "JoinToRoom"))]
    #[serde(rename = "JoinToRoom")]
    Join {
        room_id: RoomId,

        #[serde(default)]
        room_user_type: RoomUserType,
    },
    
    #[strum_discriminants(serde(rename = "RoomDisconnect"))]
    #[serde(rename = "RoomDisconnect")]
    Disconnect {
        room_id: RoomId
    },
    
    #[strum_discriminants(serde(rename = "MessageToRoom"))]
    #[serde(rename = "MessageToRoom")]
    Message {
        room_id: RoomId,
        message: Value,
    },
    
    #[strum_discriminants(serde(rename = "Play"))]
    #[serde(rename = "Play")]
    Play {
        room_id: RoomId,
        message: Value,
    },
    
    #[strum_discriminants(serde(rename = "KickUserFromroom"))]
    #[serde(rename = "KickUserFromroom")]
    Kick {
        room_id: RoomId,
        user_id: UserId,
    },
    
    #[strum_discriminants(serde(rename = "BanUserFromroom"))]
    #[serde(rename = "BanUserFromroom")]
    Ban {
        room_id: RoomId,
        user_id: UserId,
    },

    #[strum_discriminants(serde(rename = "RoomList"))]
    #[serde(rename = "RoomList")]
    List {
        #[serde(default)]
        tag: Option<String>,

        #[serde(default)]
        members: Vec<RoomInfoTypeVariant>,
    },
    
    #[strum_discriminants(serde(rename = "UpdateRoom"))]
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
    },

    #[strum_discriminants(serde(rename = "ProcessWaitingUser"))]
    #[serde(rename = "ProcessWaitingUser")]
    ProcessWaitingUser {
        room_id: RoomId,
        user_id: UserId,
        status: bool
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
