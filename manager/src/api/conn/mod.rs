

pub mod model;

#[cfg(test)]
mod test;

use std::collections::HashMap;
use std::sync::Arc;

use actix::Handler;
use actix::Actor;
use actix::Context;
use actix_broker::BrokerSubscribe;

use general::client::ClientTrait;
use general::config::YummyConfig;
use general::model::UserId;
use general::state::SendMessage;

use self::model::UserConnected;

use super::auth::model::UserDisconnectRequest;

pub struct CommunicationManager {
    config: Arc<YummyConfig>,
    users: HashMap<UserId, Arc<dyn ClientTrait + Sync + Send>>
}

impl CommunicationManager {
    pub fn new(config: Arc<YummyConfig>) -> Self {
        Self {
            config,
            users: HashMap::default()
        }
    }
}

impl Actor for CommunicationManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("CommunicationManager");
        self.subscribe_system_async::<UserConnected>(ctx);
        self.subscribe_system_async::<UserDisconnectRequest>(ctx);
        self.subscribe_system_async::<SendMessage>(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> actix::Running {
        println!("CommunicationManager stopping");
        actix::Running::Stop
    }
}

impl Handler<UserConnected> for CommunicationManager {
    type Result = ();

    #[tracing::instrument(name="UserConnected", skip(self, _ctx))]
    fn handle(&mut self, model: UserConnected, _ctx: &mut Self::Context) -> Self::Result {
        println!("UserConnected {:?}", model.user_id.get());
        self.users.insert(model.user_id, model.socket);
    }
}

impl Handler<UserDisconnectRequest> for CommunicationManager {
    type Result = ();

    #[tracing::instrument(name="UserDisconnectRequest", skip(self, _ctx))]
    fn handle(&mut self, model: UserDisconnectRequest, _ctx: &mut Self::Context) -> Self::Result {
        println!("conn:UserDisconnectRequest {:?}", model);
        self.users.remove(&model.user_id);
    }
}

impl Handler<SendMessage> for CommunicationManager {
    type Result = ();

    #[tracing::instrument(name="SendMessage", skip(self, _ctx))]
    fn handle(&mut self, model: SendMessage, _ctx: &mut Self::Context) -> Self::Result {
        println!("SendMessage");
        match self.users.get(&model.user_id) {
            Some(socket) => socket.send(model.message),
            None => println!("no socket {:?}", model.user_id.get())
        }
    }
}
