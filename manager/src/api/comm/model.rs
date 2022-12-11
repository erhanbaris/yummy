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

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct SendMessage {
    pub user_id: UserId,
    pub message: String
}

impl SendMessage {
    pub fn create<T:  Borrow<T> + Debug + Serialize + DeserializeOwned>(user_id: UserId, message: T) -> SendMessage {
        let message = serde_json::to_string(message.borrow());
        Self { user_id, message: message.unwrap_or_default() }
    }
}