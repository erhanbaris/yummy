pub mod model;

#[cfg(test)]
mod test;

use std::marker::PhantomData;
use std::sync::Arc;

use actix::Handler;
use actix::Actor;
use actix::Context;
use actix_broker::BrokerSubscribe;
use database::DatabaseTrait;

use general::config::YummyConfig;
use general::model::WebsocketMessage;
use general::model::YummyState;


use self::model::SendMessage;
use self::model::UserConnected;

use super::auth::model::UserDisconnectRequest;

pub struct CommunicationManager {
    config: Arc<YummyConfig>,
    states: Arc<YummyState>,
}

impl CommunicationManager {
    pub fn new(config: Arc<YummyConfig>, states: Arc<YummyState>) -> Self {
        Self {
            config,
            states,
        }
    }
}

impl Actor for CommunicationManager {
    type Context = Context<Self>;

    fn started(&mut self,ctx: &mut Self::Context) {
        self.subscribe_system_async::<UserDisconnectRequest>(ctx);
        self.subscribe_system_async::<SendMessage>(ctx);
    }
}

impl Handler<UserDisconnectRequest> for CommunicationManager {
    type Result = ();

    #[tracing::instrument(name="User::User disconnected", skip(self, _ctx))]
    fn handle(&mut self, user: UserDisconnectRequest, _ctx: &mut Self::Context) -> Self::Result {
        println!("connection:UserDisconnectRequest {:?}", user);
    }
}

impl Handler<SendMessage> for CommunicationManager {
    type Result = ();

    #[tracing::instrument(name="User::Send message", skip(self, _ctx))]
    fn handle(&mut self, model: SendMessage, _ctx: &mut Self::Context) -> Self::Result {
        println!("connection:SendMessage {:?}", model);

        match self.states.get_user_socket(model.user_id) {
            Some(socket) => socket.do_send(WebsocketMessage(model.message)),
            None => ()
        }
    }
}

impl Handler<UserConnected> for CommunicationManager {
    type Result = ();

    #[tracing::instrument(name="connection::UserConnected", skip(self, _ctx))]
    fn handle(&mut self, model: UserConnected, _ctx: &mut Context<Self>) -> Self::Result {

    }
}
