use std::collections::HashMap;

use serde::Serialize;
use serde::Deserialize;

use yummy_model::UserId;
use yummy_model::RoomId;
use yummy_model::RoomUserType;

use yummy_cache::state::RoomUserInformation;

macro_rules! into_impl {
    ($name: ident) => {
        impl From<$name> for String {
            fn from(source: $name) -> Self {
                serde_json::to_string(&source).unwrap()
            }
        }

        impl From<String> for $name {
            fn from(source: String) -> Self {
                serde_json::from_str(&source).unwrap()
            }
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReceiveError {
    pub status: bool,
    pub error: String
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthenticatedModel {
    pub status: bool,
    #[serde(rename = "type")]
    pub class_type: String,
    pub token: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Joined {
    #[serde(rename = "type")]
    pub class_type: String,
    pub room_name: Option<String>,
    pub users: Vec<RoomUserInformation>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRequestDeclined {
    #[serde(rename = "type")]
    pub class_type: String,
    pub room_id: RoomId
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserJoinedToRoom {
    #[serde(rename = "type")]
    pub class_type: String,
    pub user: UserId,
    pub room_id: RoomId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewRoomJoinRequest {
    #[serde(rename = "type")]
    pub class_type: String,
    pub user: UserId,
    pub room_id: RoomId,
    pub user_type: RoomUserType
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRequested {
    #[serde(rename = "type")]
    pub class_type: String,
    pub room_id: RoomId
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WaitingRoomJoinsResponse {
    #[serde(rename = "type")]
    pub class_type: String,
    pub room_id: RoomId,
    pub users: HashMap<UserId, RoomUserType>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomCreated {
    #[serde(rename = "type")]
    pub class_type: String,
    pub room_id: RoomId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDisconnectedFromRoom {
    #[serde(rename = "type")]
    pub class_type: String,
    pub user: UserId,
    pub room_id: RoomId
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisconnectedFromRoom {
    #[serde(rename = "type")]
    pub class_type: String,
    pub room_id: RoomId
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageReceivedFromRoom {
    #[serde(rename = "type")]
    pub class_type: String,
    pub user: UserId,
    pub room_id: RoomId,
    pub message: String
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeModel {
    pub status: bool,
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub device_id: Option<String>,
    pub custom_id: Option<String>,
    pub metas: serde_json::Value,
    pub user_type: i64,
    pub online: bool,
    pub insert_date: i64,
    pub last_login_date: i64,
}

into_impl!(AuthenticatedModel);
into_impl!(MeModel);
into_impl!(Joined);
into_impl!(UserJoinedToRoom);
into_impl!(RoomCreated);
into_impl!(UserDisconnectedFromRoom);
into_impl!(MessageReceivedFromRoom);