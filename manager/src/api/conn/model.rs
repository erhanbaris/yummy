use std::borrow::Borrow;
use std::fmt::Debug;
use std::sync::Arc;


use actix::Recipient;
use actix::prelude::Message;
use general::client::ClientTrait;
use serde::Serialize;
use serde::de::DeserializeOwned;
use validator::Validate;

use general::model::UserId;
use general::model::WebsocketMessage;


#[derive(Message, Validate, Debug, Clone)]
#[rtype(result = "()")]
pub struct UserConnected {
    pub user_id: UserId,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}
