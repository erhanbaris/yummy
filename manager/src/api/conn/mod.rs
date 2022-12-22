

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

pub struct ConnectionManager {
    config: Arc<YummyConfig>,
    states: YummyState,
    users: HashMap<UserId, Arc<dyn ClientTrait + Sync + Send>>
}

impl ConnectionManager {
    pub fn new(config: Arc<YummyConfig>, states: YummyState) -> Self {
        Self {
            config,
            states,
            users: HashMap::default()
        }
    }
}

#[cfg(feature = "stateless")]
mod stateless {
    use actix::{Message, Handler};
    use general::state::SendMessage;
    use actix::AsyncContext;

    use super::ConnectionManager;


    #[derive(Message, Debug, Clone)]
    #[rtype(result = "()")]
    pub struct MessageToClientReceived(pub String);
    impl general::pubsub::PubSubMessage for MessageToClientReceived {
        fn new(message: String) -> Self {
            Self(message)
        }
    }

    impl Handler<MessageToClientReceived> for ConnectionManager {
        type Result = ();
    
        #[tracing::instrument(name="MessageToClientReceived", skip(self, ctx))]
        fn handle(&mut self, model: MessageToClientReceived, ctx: &mut Self::Context) -> Self::Result {
            let message: SendMessage = match serde_json::from_str(&model.0) {
                Ok(message) => message,
                Err(error) => {
                    println!("Message parse error : {}", error.to_string());
                    return ;
                }
            };
    
            ctx.address().do_send(message);
        }
    }
}

impl Actor for ConnectionManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.subscribe_system_async::<UserConnected>(ctx);
        self.subscribe_system_async::<UserDisconnectRequest>(ctx);
        self.subscribe_system_async::<SendMessage>(ctx);

        #[cfg(feature = "stateless")]
        general::pubsub::subscribe::<stateless::MessageToClientReceived, _>(self, ctx, self.config.clone(), format!("m-{}", self.config.server_name));
    }
}

impl Handler<UserConnected> for ConnectionManager {
    type Result = ();

    #[tracing::instrument(name="UserConnected", skip(self, _ctx))]
    fn handle(&mut self, model: UserConnected, _ctx: &mut Self::Context) -> Self::Result {
        self.users.insert(model.user_id, model.socket);
    }
}

impl Handler<UserDisconnectRequest> for ConnectionManager {
    type Result = ();

    #[tracing::instrument(name="UserDisconnectRequest", skip(self, _ctx))]
    fn handle(&mut self, model: UserDisconnectRequest, _ctx: &mut Self::Context) -> Self::Result {
        println!("conn:UserDisconnectRequest {:?}", model);
        self.users.remove(&model.user_id);
    }
}

impl Handler<SendMessage> for ConnectionManager {
    type Result = ();

    #[tracing::instrument(name="SendMessage", skip(self, _ctx))]
    fn handle(&mut self, model: SendMessage, _ctx: &mut Self::Context) -> Self::Result {
        match self.users.get(&model.user_id) {
            Some(socket) => socket.send(model.message),
            None => {
                match self.states.get_user_location(model.user_id) {
                    Some(server_name) => {

                    },
                    None => println!("no socket {:?}", model.user_id.get())
                }
            }
        }
    }
}
