

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
use general::state::YummyState;

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

    fn started(&mut self,ctx: &mut Self::Context) {
        self.subscribe_system_async::<UserDisconnectRequest>(ctx);
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

        match self.users.get(&model.user_id) {
            Some(socket) => socket.send(model.message),
            None => ()
        }
    }
}
