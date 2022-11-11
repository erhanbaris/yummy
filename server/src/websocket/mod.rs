pub(crate) mod request;
pub(crate) mod response;

use std::sync::Arc;
use std::time::Instant;

use actix::Actor;
use actix::AsyncContext;
use actix::{ActorContext, Addr, Running, StreamHandler};
use actix_web::Result;
use actix_web_actors::ws;

use core::config::YummyConfig;
use crate::websocket::request::*;

pub struct GameWebsocket<M: Actor> {
    manager: Addr<M>,
    hb: Instant,
    connection_info: ConnectionInfo,
    config: Arc<YummyConfig>,
}

impl<M: Actor> GameWebsocket<M> {
    pub fn new(
        config: Arc<YummyConfig>,
        connection_info: ConnectionInfo,
        manager: Addr<M>,
    ) -> Self {
        Self {
            connection_info,
            hb: Instant::now(),
            manager,
            config,
        }
    }

    fn execute_message(&self, message: String) {
        match serde_json::from_str::<Request>(&message) {
            Ok(message) => {
                match message {
                    Request::Auth {
                        auth_type,
                        if_not_exist_create
                    } => {
                        log::debug!("Auth {:?}", auth_type);
                    }
                };
            }
            Err(error) => {
                log::error!("{:?}", error);
            }
        };
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

impl<M: Actor> Actor for GameWebsocket<M> {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::debug!("New socket started");
        self.hb(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        Running::Stop
    }
}

impl<M: Actor> StreamHandler<Result<ws::Message, ws::ProtocolError>>
    for GameWebsocket<M>
{
    fn handle(&mut self, message: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
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
            Ok(ws::Message::Text(text)) => self.execute_message(text.to_string()),
            Ok(ws::Message::Binary(bin)) => self.execute_message(std::str::from_utf8(&bin).unwrap_or_default().to_string()),
            _ => (),
        }
    }
}
