

pub mod model;

#[cfg(test)]
mod test;

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use actix::Handler;
use actix::Actor;
use actix::Context;
use actix_broker::BrokerSubscribe;

use cache::state::YummyState;
use general::client::ClientTrait;
use ::model::config::YummyConfig;
use ::model::SendMessage;
use ::model::UserId;

use actix_broker::*;

use ::model::web::Answer;
#[cfg(feature = "stateless")]
use redis::Commands;

use crate::auth::model::AuthUserDisconnect;
use crate::auth::model::ConnUserDisconnect;
use crate::auth::model::RoomUserDisconnect;

use crate::plugin::PluginExecuter;

use self::model::UserConnected;

pub struct ConnectionManager {
    #[allow(dead_code)]
    config: Arc<YummyConfig>,
    
    #[allow(dead_code)]
    states: YummyState,
    users: HashMap<UserId, Arc<dyn ClientTrait + Sync + Send>>,
    executer: Arc<PluginExecuter>,

    // Fields for stateless informations
    #[cfg(feature = "stateless")]
    redis: r2d2::Pool<redis::Client>
}

impl ConnectionManager {
    pub fn new(config: Arc<YummyConfig>, states: YummyState, executer: Arc<PluginExecuter>, #[cfg(feature = "stateless")] redis: r2d2::Pool<redis::Client>) -> Self {
        Self {
            config,
            states,
            users: HashMap::default(),
            executer,

            #[cfg(feature = "stateless")] redis
        }
    }
}

#[cfg(feature = "stateless")]
mod stateless {
    use actix::{Message, Handler};
    use model::SendMessage;
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
                    println!("Message parse error : {}", error);
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
        self.subscribe_system_async::<ConnUserDisconnect>(ctx);
        self.subscribe_system_async::<SendMessage>(ctx);

        #[cfg(feature = "stateless")]
        general::pubsub::subscribe::<stateless::MessageToClientReceived, _>(self, ctx, self.config.clone(), format!("m-{}", self.config.server_name));
    }
}

impl Handler<UserConnected> for ConnectionManager {
    type Result = ();

    #[tracing::instrument(name="UserConnected", skip(self, _ctx))]
    #[macros::plugin_api(name="user_connected", no_return=true)]
    fn handle(&mut self, model: UserConnected, _ctx: &mut Self::Context) -> Self::Result {
        self.users.insert(model.user_id.deref().clone(), model.socket.clone());
    }
}

impl Handler<ConnUserDisconnect> for ConnectionManager {
    type Result = ();

    #[tracing::instrument(name="ConnUserDisconnect", skip(self, _ctx))]
    #[macros::plugin_api(name="user_disconnected", no_return=true)]
    fn handle(&mut self, model: ConnUserDisconnect, _ctx: &mut Self::Context) -> Self::Result {
        let user_id = match model.auth.deref() {
            Some(user) => &user.user,
            None => {
                if model.send_message {
                    model.socket.send(Answer::fail(model.request_id).into());
                }
                return
            }
        };

        let user_removed = self.users.remove(user_id);

        if user_removed.is_none() {
            if model.send_message {
                model.socket.send(Answer::fail(model.request_id).into());
            }
            return;
        }
        
        if model.send_message {
            model.socket.send(Answer::success(model.request_id).into());
        }
        
        self.issue_system_async(RoomUserDisconnect {
            request_id: model.request_id,
            auth: model.auth.clone(),
            socket: model.socket.clone()
        });

        self.issue_system_async(AuthUserDisconnect {
            request_id: model.request_id,
            auth: model.auth.clone(),
            socket: model.socket.clone()
        });
    }
}

impl Handler<SendMessage> for ConnectionManager {
    type Result = ();

    #[tracing::instrument(name="SendMessage", skip(self, _ctx))]
    fn handle(&mut self, model: SendMessage, _ctx: &mut Self::Context) -> Self::Result {
        #[allow(clippy::single_match)]
        match self.users.get(model.user_id.as_ref()) {
            Some(socket) => socket.send(model.message),
            None => {
                #[cfg(feature = "stateless")]
                match self.states.get_user_location(model.user_id.clone()) {
                    Some(server_name) => {
                        if let Ok(mut redis) = self.redis.get() {
                            if let Ok(message) = serde_json::to_string(&model) {
                                redis.publish::<_, _, i32>(format!("m-{}", server_name), message).unwrap_or_default();
                            }
                        }
                    },
                    None => println!("no socket {:?}", model.user_id.get())
                }
            }
        }
    }
}
