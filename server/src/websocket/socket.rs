use crate::auth::UserJwt;
use crate::config::YummyConfig;
use crate::manager::GameManagerTrait;
use actix::Actor;
use crate::model::ConnectionId;
use actix::AsyncContext;
use actix::{ActorContext, Addr, Running, StreamHandler};
use actix_web::Result;
use actix_web_actors::ws;
use serde::Deserialize;

use std::fmt::Debug;
use std::sync::Arc;
use std::time::Instant;

use actix::{
    prelude::{Message, Recipient},
};
#[derive(Message)]
#[rtype(result = "()")]
pub struct WebsocketMessage(pub String);

pub trait SocketTrait {
    type Message;
    fn do_send(&self, msg: Self::Message);
}

pub struct RealSocket<R> {
    pub _marker: std::marker::PhantomData<R>,
    pub socket: Recipient<WebsocketMessage>,
}

impl<R> SocketTrait for RealSocket<R>
where
    std::string::String: std::convert::From<R>,
{
    type Message = R;
    fn do_send(&self, msg: Self::Message) {
        self.socket.do_send(WebsocketMessage(msg.into()));
    }
}

unsafe impl<Response> Send for RealSocket<Response> {}
unsafe impl<Response> Sync for RealSocket<Response> {}


pub struct GameWebsocket<M: Actor + GameManagerTrait> {
    user: UserJwt,
    connection_id: ConnectionId,
    manager: Addr<M>,
    hb: Instant,
    valid_user: bool,
    config: Arc<YummyConfig>
}

impl<M: Actor + GameManagerTrait> GameWebsocket<M>
{
    pub fn new(config: Arc<YummyConfig>, connection_id: usize, user: UserJwt, manager: Addr<M>, valid_user: bool) -> Self {
        Self {
            connection_id: ConnectionId(connection_id),
            user,
            hb: Instant::now(),
            manager,
            valid_user,
            config
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(self.config.heartbeat_interval, |act, ctx| {
            if Instant::now().duration_since(act.hb) > act.config.client_timeout {
                log::debug!("Disconnecting failed heartbeat, {:?}", act.hb);
                ctx.stop();
                return;
            }

            ctx.ping(b"PING");
        });
    }
}

impl<M: Actor + GameManagerTrait> Actor for GameWebsocket<M> {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::debug!("New socket started");
        if !self.valid_user {
            log::debug!("Invalid user");
            ctx.close(Some(actix_web_actors::ws::CloseReason::from(actix_web_actors::ws::CloseCode::Invalid)));
            ctx.stop();
            return;
        }

        if cfg!(not(feature="test")) {
            self.hb(ctx);
        }
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        if self.valid_user {
            log::debug!("Stopping socket ({:?})", self.connection_id);
        }
        Running::Stop
    }
}


/// Handler for ws::Message message
impl<M: Actor + GameManagerTrait> StreamHandler<Result<ws::Message, ws::ProtocolError>> for GameWebsocket<M> {
    fn handle(&mut self, message: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        log::info!("RECEIVE:{:?}:{:?}", self.connection_id, message);

        match message {
            Ok(ws::Message::Close(reason)) => {
                log::debug!("Stop: {:?}", reason);
                ctx.stop();
            }
            Ok(ws::Message::Ping(message)) => {
                self.hb = Instant::now();
                ctx.pong(&message);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(_)) => (),
            Ok(ws::Message::Binary(_)) => (),
            _ => (),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct ConnectionInfo {
    #[serde(default)]
    pub id: Option<usize>,
    pub key: String,
}

