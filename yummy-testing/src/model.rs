/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::collections::HashMap;

use serde::Serialize;
use serde::Deserialize;

use yummy_model::UserId;
use yummy_model::RoomId;
use yummy_model::RoomUserType;

use yummy_model::state::RoomUserInformation;

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* **************************************************************************************************************** */
macro_rules! into_impl {
    ($name: ident) => {
        impl From<$name> for String {
            fn from(source: $name) -> Self {
                serde_json::to_string(&source).unwrap()
            }
        }

        impl From<String> for $name {
            fn from(source: String) -> Self {
                println!("{}", &source);
                serde_json::from_str(&source).unwrap()
            }
        }
    }
}

/* **************************************************************************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
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
    pub room_name: Option<String>,
    pub users: Vec<RoomUserInformation>,
    pub metas: serde_json::Value
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRequestDeclined {
    pub room_id: RoomId,
    pub result: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserJoinedToRoom {
    pub user_id: UserId,
    pub room_id: RoomId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewRoomJoinRequest {
    #[serde(rename = "type")]
    pub class_type: String,
    pub user_id: UserId,
    pub room_id: RoomId,
    pub user_type: RoomUserType
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WaitingRoomJoinsResponse {
    pub room_id: RoomId,
    pub users: HashMap<UserId, RoomUserType>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomCreated {
    pub room_id: RoomId
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDisconnectedFromRoom {
    pub user_id: UserId,
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
    pub user_id: UserId,
    pub room_id: RoomId,
    pub message: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRequested {
    pub room_id: RoomId,
    pub result: String
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

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* **************************************************************************************************************** */
into_impl!(AuthenticatedModel);
into_impl!(MeModel);
into_impl!(Joined);
into_impl!(UserJoinedToRoom);
into_impl!(RoomCreated);
into_impl!(UserDisconnectedFromRoom);
into_impl!(MessageReceivedFromRoom);

/* **************************************************************************************************************** */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */