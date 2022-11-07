use std::sync::Arc;

use actix::{Context, Actor, Handler};

use crate::{manager::{GameManagerTrait, Connect}, websocket::socket::SocketTrait, model::ConnectionId, config::YummyConfig};


pub struct GameManager {
    config: Arc<YummyConfig>
}

impl GameManager {
    pub fn new(config: Arc<YummyConfig>) -> Self {
        Self {
            config
        }
    } 
}

impl GameManagerTrait for GameManager {
    type ConnectResponse = ();
}

impl Actor for GameManager {
    type Context = Context<Self>;
}

impl SocketTrait for GameManager {
    type Message = ();
    fn do_send(&self, _: Self::Message) {}
}

impl Handler<Connect<<Self as GameManagerTrait>::ConnectResponse>> for GameManager {
    type Result = ConnectionId;

    fn handle(&mut self, message: Connect<<Self as GameManagerTrait>::ConnectResponse>, ctx: &mut Context<Self>) -> Self::Result {
        ConnectionId::default()
    }
}