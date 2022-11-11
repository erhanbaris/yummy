pub mod api;

use actix::{Actor, Context};
use database::Pool;

use core::config::YummyConfig;
use std::sync::Arc;

pub struct GameManager {
    config: Arc<YummyConfig>,
    database: Arc<Pool>
}

impl GameManager {
    pub fn new(config: Arc<YummyConfig>, database: Arc<Pool>) -> Self {
        Self {
            config,
            database
        }
    } 
}

impl Actor for GameManager {
    type Context = Context<Self>;
}
