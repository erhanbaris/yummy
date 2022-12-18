use std::borrow::Borrow;
use std::fmt::Debug;


use actix::Recipient;
use actix::prelude::Message;
use serde::Serialize;
use serde::de::DeserializeOwned;
use validator::Validate;

use general::model::UserId;
use general::model::WebsocketMessage;


#[derive(Message, Validate, Debug)]
#[rtype(result = "()")]
pub struct UserConnected {
    pub user_id: UserId,
    pub socket: Recipient<WebsocketMessage>
}
