use database::model::UserInformationModel;
use general::auth::{UserJwt};
use general::model::RoomId;
use serde::Serialize;
use serde::Deserialize;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Response {
    None,
    UserInformation(UserInformationModel),
    RoomInformation(RoomId),
}
