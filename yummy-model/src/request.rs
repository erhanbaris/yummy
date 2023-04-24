use std::collections::HashMap;

use serde_json::Value;
use strum_macros::EnumDiscriminants;
use crate::state::RoomInfoTypeVariant;

use crate::password::Password;
use crate::{UserId, UserType, CreateRoomAccessType, RoomId, RoomUserType, meta::{MetaType, RoomMetaAccess, UserMetaAccess, MetaAction}};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, EnumDiscriminants, Debug)]
#[strum_discriminants(name(RequestAuthTypeVariant), derive(Deserialize, Serialize))]
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
#[strum_discriminants(name(RequestUserTypeVariant), derive(Deserialize, Serialize))]
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
#[strum_discriminants(name(RequestRoomTypeVariant), derive(Deserialize, Serialize))]
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

    #[strum_discriminants(serde(rename = "GetRoom"))]
    #[serde(rename = "GetRoom")]
    GetRoom {
        room_id: RoomId,

        #[serde(default)]
        members: Vec<RoomInfoTypeVariant>,
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
    
    #[strum_discriminants(serde(rename = "KickUserFromRoom"))]
    #[serde(rename = "KickUserFromRoom")]
    Kick {
        room_id: RoomId,
        user_id: UserId,
    },
    
    #[strum_discriminants(serde(rename = "BanUserFromRoom"))]
    #[serde(rename = "BanUserFromRoom")]
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

    #[strum_discriminants(serde(rename = "WaitingRoomJoins"))]
    #[serde(rename = "WaitingRoomJoins")]
    WaitingRoomJoins {
        room_id: RoomId
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

impl From<RequestUserTypeVariant> for &'static str {
    fn from(value: RequestUserTypeVariant) -> Self {
        match value {
            RequestUserTypeVariant::Me => "Me",
            RequestUserTypeVariant::Get => "GetUser",
            RequestUserTypeVariant::Update => "UpdateUser",
        }
    }
}


impl From<RequestAuthTypeVariant> for &'static str {
    fn from(value: RequestAuthTypeVariant) -> Self {
        match value {
            RequestAuthTypeVariant::Email => "AuthEmail",
            RequestAuthTypeVariant::DeviceId => "AuthDeviceId",
            RequestAuthTypeVariant::CustomId => "AuthCustomId",
            RequestAuthTypeVariant::Refresh => "RefreshToken",
            RequestAuthTypeVariant::Restore => "RestoreToken",
            RequestAuthTypeVariant::Logout => "Logout",
        }
    }
}


impl From<RequestRoomTypeVariant> for &'static str {
    fn from(value: RequestRoomTypeVariant) -> Self {
        match value {
            RequestRoomTypeVariant::Create => "CreateRoom",
            RequestRoomTypeVariant::GetRoom => "GetRoom",
            RequestRoomTypeVariant::Join => "JoinToRoom",
            RequestRoomTypeVariant::Disconnect => "RoomDisconnect",
            RequestRoomTypeVariant::Message => "MessageToRoom",
            RequestRoomTypeVariant::Play => "Play",
            RequestRoomTypeVariant::Kick => "KickUserFromRoom",
            RequestRoomTypeVariant::Ban => "BanUserFromRoom",
            RequestRoomTypeVariant::List => "RoomList",
            RequestRoomTypeVariant::Update => "UpdateUser",
            RequestRoomTypeVariant::WaitingRoomJoins => "WaitingRoomJoins",
            RequestRoomTypeVariant::ProcessWaitingUser => "ProcessWaitingUser",
        }
    }
}