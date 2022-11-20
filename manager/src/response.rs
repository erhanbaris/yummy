use database::model::PrivateUserModel;
use database::model::PublicUserModel;
use general::auth::{UserJwt};
use serde::Serialize;
use serde::Deserialize;

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    None,
    Auth(String, UserJwt),
    UserPrivateInfo(PrivateUserModel),
    UserPublicInfo(PublicUserModel),
}
