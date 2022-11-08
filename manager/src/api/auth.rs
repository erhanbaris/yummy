use actix::{Context, Handler, Recipient};
use actix::prelude::Message;

use core::jwt::UserJwt;
use core::model::{SessionId, WebsocketMessage};

use crate::GameManager;

#[derive(Message)]
#[rtype(result = "SessionId")]
pub struct Auth {
    pub connection_id: SessionId,
    pub user: UserJwt,
    pub socket: Box<Recipient<WebsocketMessage>>,
}

impl Handler<Auth> for GameManager {
    type Result = SessionId;

    fn handle(&mut self, _: Auth, _: &mut Context<Self>) -> Self::Result {
        SessionId::default()
    }
}
