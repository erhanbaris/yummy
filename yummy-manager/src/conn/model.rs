use std::fmt::Debug;
use std::sync::Arc;

use actix::prelude::Message;
use yummy_general::client::ClientTrait;
use validator::Validate;

use yummy_model::UserId;


#[derive(Message, Validate, Debug, Clone)]
#[rtype(result = "()")]
pub struct UserConnected {
    pub user_id: Arc<UserId>,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}
