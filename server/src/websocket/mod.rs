pub(crate) mod request;
pub(crate) mod response;

use core::model::WebsocketMessage;
use core::web::GenericAnswer;
use std::sync::Arc;
use std::time::Instant;

use actix::Actor;
use actix::AsyncContext;
use actix::ActorFutureExt;
use actix::ContextFutureSpawner;
use actix::Handler;
use actix::WrapFuture;
use actix::{ActorContext, Addr, Running, StreamHandler, fut};
use actix_web::Result;
use actix_web_actors::ws;
use database::auth::AuthStoreTrait;
use manager::api::auth::AuthManager;
use manager::api::auth::EmailAuth;
use secrecy::SecretString;

use core::config::YummyConfig;
use crate::websocket::request::*;

pub struct GameWebsocket<A: AuthStoreTrait + Unpin + 'static> {
    auth: Addr<AuthManager<A>>,
    hb: Instant,
    connection_info: ConnectionInfo,
    config: Arc<YummyConfig>,
}

macro_rules! spawn_future {
    ($fu: expr, $self: expr, $ctx: expr) => {
        $fu
        .into_actor($self)
        .then(|res, _, ctx| {

            let response = match res {
                Ok(result) => match result {
                    Ok(result) => String::from(GenericAnswer {
                        status: true,
                        result: Some(result),
                    }),
                    Err(error) => String::from(GenericAnswer {
                        status: false,
                        result: Some(error.to_string()),
                    })
                },
                Err(_) => String::from(GenericAnswer {
                    status: false,
                    result: Some("Unexpected internal error"),
                })
            };

            ctx.text(String::from(response));
            fut::ready(())
            
        })
        .spawn($ctx)
    };
}

impl<A: AuthStoreTrait + Unpin + 'static> GameWebsocket<A> {
    pub fn new(
        config: Arc<YummyConfig>,
        connection_info: ConnectionInfo,
        auth: Addr<AuthManager<A>>,
    ) -> Self {
        Self {
            connection_info,
            hb: Instant::now(),
            auth,
            config,
        }
    }

    fn auth(&self, auth_type: AuthType, if_not_exist_create: bool, ctx: &mut ws::WebsocketContext<Self>) {
        match auth_type {
            AuthType::Email { email, password } => {
                spawn_future!(self.auth.send(EmailAuth { email, password: SecretString::new(password), if_not_exist_create }), self, ctx);
            },
            _ => todo!("Not implemented")
        }
    }

    fn execute_message(&self, message: String, ctx: &mut ws::WebsocketContext<Self>) {
        match serde_json::from_str::<Request>(&message) {
            Ok(message) => {
                match message {
                    Request::Auth { auth_type, if_not_exist_create } => self.auth(auth_type, if_not_exist_create, ctx)
                };
            }
            Err(error) => {
                log::error!("{:?}", error);
                ctx.text(r#"{"status": false,"result": "Wrong message format"}"#);
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

impl<A: AuthStoreTrait + Unpin + 'static> Actor for GameWebsocket<A> {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::debug!("New socket started");
        self.hb(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        Running::Stop
    }
}

impl<A: AuthStoreTrait + Unpin + 'static> StreamHandler<Result<ws::Message, ws::ProtocolError>>
    for GameWebsocket<A>
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
            Ok(ws::Message::Text(text)) => self.execute_message(text.to_string(), ctx),
            Ok(ws::Message::Binary(bin)) => self.execute_message(std::str::from_utf8(&bin).unwrap_or_default().to_string(), ctx),
            _ => (),
        }
    }
}

impl<A: AuthStoreTrait + Unpin + 'static> Handler<WebsocketMessage> for GameWebsocket<A> {
    type Result = ();

    fn handle(&mut self, message: WebsocketMessage, ctx: &mut Self::Context) {
        log::info!("SEND:{:?}", message.0);
        ctx.text(message.0);
    }
}
