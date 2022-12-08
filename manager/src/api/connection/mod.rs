pub mod model;

#[cfg(test)]
mod test;

use std::time::Duration;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

use actix::Handler;
use actix::Actor;
use actix::Context;
use actix_broker::BrokerSubscribe;
use database::DatabaseTrait;

use general::config::YummyConfig;
use general::model::YummyState;


use self::model::UserConnected;

use super::auth::model::UserDisconnectRequest;

pub struct ConnectionManager<DB: DatabaseTrait + ?Sized> {
    config: Arc<YummyConfig>,
    states: Arc<YummyState>,
    _marker: PhantomData<DB>,
}

impl<DB: DatabaseTrait + ?Sized> ConnectionManager<DB> {
    pub fn new(config: Arc<YummyConfig>, states: Arc<YummyState>) -> Self {
        Self {
            config,
            states,
            _marker: PhantomData,
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Actor for ConnectionManager<DB> {
    type Context = Context<Self>;

    fn started(&mut self,ctx: &mut Self::Context) {
        self.subscribe_system_async::<UserDisconnectRequest>(ctx);
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<UserDisconnectRequest> for ConnectionManager<DB> {
    type Result = ();

    #[tracing::instrument(name="User::User disconnected", skip(self, _ctx))]
    fn handle(&mut self, user: UserDisconnectRequest, _ctx: &mut Self::Context) -> Self::Result {
        println!("connection:UserDisconnectRequest {:?}", user);
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<UserConnected> for ConnectionManager<DB> {
    type Result = ();

    #[tracing::instrument(name="connection::UserConnected", skip(self, _ctx))]
    fn handle(&mut self, model: UserConnected, _ctx: &mut Context<Self>) -> Self::Result {

    }
}
