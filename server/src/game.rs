use std::sync::Arc;

use actix::{Context, Actor, Handler};

use crate::{manager::{GameManagerTrait, Connect}, model::SessionId, config::YummyConfig};


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

impl Handler<Connect> for GameManager {
    type Result = SessionId;

    fn handle(&mut self, _: Connect, _: &mut Context<Self>) -> Self::Result {
        SessionId::default()
    }
}