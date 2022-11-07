use crate::auth::UserJwt;
use crate::config::YummyConfig;
use crate::manager::GameManagerTrait;
use crate::manager::*;
use crate::model::ConnectionId;
use crate::model::RoomId;
use actix::ActorFutureExt;
use actix::AsyncContext;
use actix::ContextFutureSpawner;
use actix::WrapFuture;
use actix::{fut, Actor, ActorContext, Addr, Handler, Running, StreamHandler};
use actix_web::Result;
use actix_web_actors::ws;
use serde::Deserialize;
use serde::Serialize;

use std::fmt::Debug;
use std::sync::Arc;
use std::time::Instant;

use actix::{
    prelude::{Message, Recipient},
};
#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

pub trait SocketTrait {
    type Message;
    fn do_send(&self, msg: Self::Message);
}

pub struct RealSocket<R> {
    pub _marker: std::marker::PhantomData<R>,
    pub socket: Recipient<WsMessage>,
}

impl<R> SocketTrait for RealSocket<R>
where
    std::string::String: std::convert::From<R>,
{
    type Message = R;
    fn do_send(&self, msg: Self::Message) {
        self.socket.do_send(WsMessage(msg.into()));
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
where
    std::string::String: std::convert::From<<M as GameManagerTrait>::Response>,
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

    fn execute_message(&self, message: String) {
        match serde_json::from_str::<Request<M::Move, M::Action, M::NewGameConfig>>(&message) {
            Ok(message) => {
                match message {
                    Request::Message { message } => {
                        self.manager.do_send(MessageToRoom { connection_id: self.connection_id, message });
                    }
                    Request::Debug => {
                        println!("Debug")
                    }
                    Request::ExitFromRoom => {
                        self.manager.do_send(ExitFromRoom { connection_id: self.connection_id });
                    }
                    Request::Kick { room, player } => self.manager.do_send(KickPlayerFromRoom { connection_id: ConnectionId(player), room_id: RoomId(room) }),
                    Request::Play { room, player_move } => self.manager.do_send(UserPlay {
                        connection_id: self.connection_id,
                        room_id: RoomId(room),
                        player_move,
                    }),
                    Request::Action { room, action } => self.manager.do_send(UserAction {
                        connection_id:self.connection_id,
                        room_id: RoomId(room),
                        action,
                    }),
                    Request::NewGame { game_type, config } => self.manager.do_send(NewGame {
                        connection_id:self.connection_id,
                        game_type,
                        config,
                    }),
                    Request::Ready { room } => self.manager.do_send(UserReady { connection_id: self.connection_id, room_id: RoomId(room) }),
                    Request::Join { room } => self.manager.do_send(JoinToRoom { connection_id: self.connection_id, room }),
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
                println!("Disconnecting failed heartbeat, {:?}", act.hb);
                ctx.stop();
                return;
            }

            ctx.ping(b"PING");
        });
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum Request<C, A, I>
where
    C: Serialize + Debug + Send,
    A: Serialize + Debug + Send,
    I: Serialize + Debug + Send,
{
    Message { message: String },
    NewGame { game_type: NewGameType, config: Option<I> },
    ExitFromRoom,
    Ready { room: usize },
    Join { room: JoinToRoomType },
    Play { room: usize, player_move: C },
    Action { room: usize, action: A },
    Kick { room: usize, player: usize },
    Debug,
}

impl<C, A, I> From<Request<C, A, I>> for String
where
    C: Serialize + Send + Debug,
    A: Serialize + Send + Debug,
    I: Serialize + Send + Debug,
{
    fn from(request: Request<C, A, I>) -> String {
        serde_json::to_string(&request).unwrap_or_default()
    }
}

impl<M: Actor + GameManagerTrait> Actor for GameWebsocket<M>
where
    std::string::String: std::convert::From<<M as GameManagerTrait>::Response>,
{
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Started");
        if !self.valid_user {
            println!("Invalid user");
            ctx.close(Some(actix_web_actors::ws::CloseReason::from(actix_web_actors::ws::CloseCode::Invalid)));
            ctx.stop();
            return;
        }

        if cfg!(not(feature="test")) {
            self.hb(ctx);
        }

        let addr = ctx.address();
        self.manager
            .send(Connect::<M::Response> {
                connection_id: self.connection_id,
                user: self.user.clone(),
                socket: Box::new(RealSocket {
                    _marker: std::marker::PhantomData,
                    socket: addr.recipient(),
                }),
            })
            .into_actor(self)
            .then(|res, data, ctx| {
                match res {
                    Ok(connection_id) => match connection_id.empty() {
                        true => {
                            println!("Connection id is 0");
                            ctx.stop();
                        }
                        false => {
                            data.connection_id = connection_id;
                        }
                    },
                    _ => {
                        println!("Start return message not accepted");
                        ctx.stop();
                    }
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        if self.valid_user {
            println!("Stopping socket ({:?})", self.connection_id);
            self.manager.do_send(StartDisconnectProcedure {
                connection_id: self.connection_id,
                apply_now: false,
            });
        }
        Running::Stop
    }
}

impl<M: Actor + GameManagerTrait> Handler<WsMessage> for GameWebsocket<M>
where
    std::string::String: std::convert::From<<M as GameManagerTrait>::Response>,
{
    type Result = ();

    fn handle(&mut self, message: WsMessage, ctx: &mut Self::Context) {
        log::info!("SEND:{:?}:{:?}", self.connection_id, message.0);
        ctx.text(message.0);
    }
}

/// Handler for ws::Message message
impl<M: Actor + GameManagerTrait> StreamHandler<Result<ws::Message, ws::ProtocolError>> for GameWebsocket<M>
where
    std::string::String: std::convert::From<<M as GameManagerTrait>::Response>,
{
    fn handle(&mut self, message: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        log::info!("RECEIVE:{:?}:{:?}", self.connection_id, message);

        match message {
            Ok(ws::Message::Close(reason)) => {
                println!("Stop: {:?}", reason);
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

#[derive(Debug, Deserialize, Default)]
pub struct ConnectionInfo {
    #[serde(default)]
    pub id: Option<usize>,
    pub key: String,
}

