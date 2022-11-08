pub mod api;

use std::fmt::Debug;

use actix::{Actor, Context, Handler};
use api::auth::Auth;
use serde::{Serialize, de::DeserializeOwned};

pub trait GameManagerTrait: Actor<Context = Context<Self>> + Handler<Auth>
where
    Self: std::marker::Sized,
{
    type ConnectResponse: Serialize + DeserializeOwned + Send + Debug;
}

use core::config::YummyConfig;
use std::sync::Arc;

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
