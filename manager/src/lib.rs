pub mod api;

use std::fmt::Debug;

use actix::{Actor, Context, Handler};
use api::auth::EmailAuth;
use database::Pool;
use database::auth::AuthStore;
use serde::{Serialize, de::DeserializeOwned};

pub trait GameManagerTrait: Actor<Context = Context<Self>> + Handler<EmailAuth>
where
    Self: std::marker::Sized,
{
    type EmailAuthResponse: Serialize + DeserializeOwned + Send + Debug;
}

use core::config::YummyConfig;
use std::sync::Arc;

pub struct GameManager {
    config: Arc<YummyConfig>,
    database: Arc<Pool>
}

impl GameManager {
    pub fn new(config: Arc<YummyConfig>) -> anyhow::Result<Self> {
        let database = Arc::new(database::create_connection(&config.database_url)?);
        let mut connection = database.clone().get()?;
        database::create_database(&mut connection)?;

        Ok(Self {
            config,
            database
        })
    } 
}

impl GameManagerTrait for GameManager {
    type EmailAuthResponse = ();
}

impl Actor for GameManager {
    type Context = Context<Self>;
}
