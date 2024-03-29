/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* **************************************************************************************************************** */
#[cfg(test)]
mod tests;

/* **************************************************************************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use actix::Recipient;
use actix_web::HttpRequest;
use actix_web::web::Data;
use actix_web::web::Payload;
use yummy_database::DatabaseTrait;
use yummy_model::WebsocketMessage;
use yummy_model::auth::ApiIntegration;
use yummy_model::auth::UserAuth;
use yummy_general::client::ClientTrait;
use yummy_general::error::YummyError;
use yummy_model::UserAuthenticated;
use yummy_model::request::Request;
use yummy_model::web::GenericAnswer;
use yummy_manager::auth::model::StartUserTimeout;
use yummy_manager::room::RoomManager;
use yummy_manager::user::UserManager;
use std::borrow::Cow;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Instant;

use yummy_general::client::EmptyClient;
use actix::Actor;
use actix::AsyncContext;
use actix::Handler;
use actix::{ActorContext, Addr, Running, StreamHandler};
use actix_web::Result;
use actix_web_actors::ws;
use yummy_manager::auth::AuthManager;

use yummy_model::config::YummyConfig;
use crate::api::process_auth;
use crate::api::process_user;

use super::ProcessResult;
use super::process_room;

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
pub struct GameWebsocket<DB: DatabaseTrait + ?Sized + Unpin + 'static> {
    auth_manager: Addr<AuthManager<DB>>,
    user_manager: Addr<UserManager<DB>>,
    room_manager: Addr<RoomManager<DB>>,
    hb: Instant,
    user_auth: Arc<Option<UserAuth>>,
    config: Arc<YummyConfig>,
    client: Arc<dyn ClientTrait + Sync + Send>
}

#[derive(Debug)]
struct GameWebsocketClient {
    sender: Recipient<WebsocketMessage>,
    auth: Recipient<UserAuthenticated>
}

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* **************************************************************************************************************** */
pub async fn websocket_endpoint<DB: DatabaseTrait + Unpin + 'static>(req: HttpRequest, stream: Payload, config: Data<Arc<YummyConfig>>, auth_manager: Data<Addr<AuthManager<DB>>>, user_manager: Data<Addr<UserManager<DB>>>, room_manager: Data<Addr<RoomManager<DB>>>, _: ApiIntegration) -> Result<actix_web::HttpResponse, YummyError> {
    let config = config.get_ref();

    ws::start(GameWebsocket::new(config.clone(),
        auth_manager.get_ref().clone(),
        user_manager.get_ref().clone(),
        room_manager.get_ref().clone()),
        &req, stream)
        .map_err(YummyError::from)
}

/* **************************************************************************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* **************************************************************************************************************** */
impl<DB: DatabaseTrait + ?Sized + Unpin + 'static> GameWebsocket<DB> {
    pub fn new(
        config: Arc<YummyConfig>,
        auth: Addr<AuthManager<DB>>,
        user: Addr<UserManager<DB>>,
        room: Addr<RoomManager<DB>>,
    ) -> Self {
        Self {
            hb: Instant::now(),
            auth_manager: auth,
            user_manager: user,
            room_manager: room,
            config,
            user_auth: Arc::new(None),
            client: Arc::new(EmptyClient::default())
        }
    }

    #[tracing::instrument(name="execute_message", skip(self, ctx))]
    fn execute_message(&mut self, message: String, ctx: &mut ws::WebsocketContext<Self>) -> ProcessResult {
        let message = match serde_json::from_str::<Request>(&message) {
            Ok(message) => message,
            Err(error) => {
                println!("{}", error);
                ctx.text(serde_json::to_string(&GenericAnswer::fail(None, Cow::Borrowed(""), "Wrong message format")).unwrap());
                return Ok(());
            }
        };

        let user_info = self.user_auth.clone();
        let socket = self.client.clone();

        let validation = match message {
            Request::Auth { request_id, auth_type } => process_auth(request_id, auth_type, self.auth_manager.clone(), user_info, socket),
            Request::User { request_id, user_type } => process_user(request_id, user_type, self.user_manager.clone(), user_info, socket),
            Request::Room { request_id, room_type } => process_room(request_id, room_type, self.room_manager.clone(), user_info, socket),
        };

        if let Err((request_id, request_type, error)) = validation {
            ctx.text(serde_json::to_string(&GenericAnswer::fail(request_id, Cow::Borrowed(request_type), error.to_string())).unwrap())
        }

        Ok(())
    }

    #[tracing::instrument(name="HB", skip(self, ctx))]
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(self.config.heartbeat_interval, |act, ctx| {
            if Instant::now().duration_since(act.hb) > act.config.heartbeat_timeout {
                println!("Disconnecting failed heartbeat, {:?}", act.hb);
                ctx.stop();
                return;
            }
            ctx.ping(b"PING");
        });
    }
}

impl GameWebsocketClient {
    pub fn new<DB: DatabaseTrait + ?Sized + Unpin + 'static>(address: Addr<GameWebsocket<DB>>) -> Self {
        Self {
            sender: address.clone().recipient(),
            auth: address.recipient()
        }
    }
}

/* **************************************************************************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* **************************************************************************************************************** */
impl<DB: DatabaseTrait + ?Sized + Unpin + 'static> Actor for GameWebsocket<DB> {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::debug!("New socket started");
        self.client = Arc::new(GameWebsocketClient::new(ctx.address()));
        self.hb(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.auth_manager.do_send(StartUserTimeout {
            auth: self.user_auth.clone(),
            socket: self.client.clone()
        });

        Running::Stop
    }
}

impl<DB: DatabaseTrait + ?Sized + Unpin + 'static> StreamHandler<Result<ws::Message, ws::ProtocolError>>
    for GameWebsocket<DB>
{
    #[tracing::instrument(name="Message", skip(self, ctx))]
    fn handle(&mut self, message: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {

        let result = match message {
            Ok(ws::Message::Close(reason)) => {
                log::debug!("Stop: {:?}", reason);
                ctx.stop();
                Ok(())
            }
            Ok(ws::Message::Ping(message)) => {
                self.hb = Instant::now();
                ctx.pong(&message);
                Ok(())
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
                Ok(())
            }
            Ok(ws::Message::Text(text)) => self.execute_message(text.to_string(), ctx),
            Ok(ws::Message::Binary(bin)) => self.execute_message(std::str::from_utf8(&bin).unwrap_or_default().to_string(), ctx),
            _ => Ok(()),
        };

        if let Err((request_id, request_type, error)) = result {
            ctx.text(String::from(GenericAnswer::fail(request_id, Cow::Borrowed(request_type), error.to_string())));
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + Unpin + 'static> Handler<WebsocketMessage> for GameWebsocket<DB> {
    type Result = ();
    
    #[tracing::instrument(name="WebsocketMessage", skip(self, ctx))]
    fn handle(&mut self, message: WebsocketMessage, ctx: &mut Self::Context) {
        ctx.text(message.0);
    }
}

impl<DB: DatabaseTrait + ?Sized + Unpin + 'static> Handler<UserAuthenticated> for GameWebsocket<DB> {
    type Result = ();
    
    #[tracing::instrument(name="UserAuthenticated", skip(self, _ctx))]
    fn handle(&mut self, model: UserAuthenticated, _ctx: &mut Self::Context) {
        log::info!("AUTH:{:?}", model.0);
        self.user_auth = Arc::new(Some(UserAuth {
            user: model.0.id.deref().clone(),
            session: model.0.session
        }));
    }
}

impl ClientTrait for GameWebsocketClient {
    fn send(&self, message: String) {
        self.sender.do_send(WebsocketMessage(message));
    }

    fn authenticated(&self, user: yummy_model::auth::UserJwt) {
        self.auth.do_send(UserAuthenticated(user));
    }
}
/* **************************************************************************************************************** */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */