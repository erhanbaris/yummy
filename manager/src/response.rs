use database::model::PrivateUserModel;
use general::auth::{UserJwt};
use serde::Serialize;
use serde::Deserialize;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Response {
    None,
    Auth(String, UserJwt),
    UserPrivateInfo(PrivateUserModel),
}
