pub(crate) mod request;
pub(crate) mod response;
#[cfg(test)]
mod client;

use actix_web::HttpRequest;
use actix_web::web::Data;
use actix_web::web::Query;
use actix_web::web::Payload;
use general::error::YummyError;
use general::model::WebsocketMessage;
use general::web::GenericAnswer;
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
use manager::api::auth::DeviceIdAuth;
use manager::api::auth::EmailAuth;
use manager::api::auth::RefreshToken;
use validator::Validate;

use general::config::YummyConfig;
use crate::websocket::request::*;

pub async fn websocket_endpoint<A: AuthStoreTrait + Unpin + 'static>(req: HttpRequest, stream: Payload, config: Data<Arc<YummyConfig>>, manager: Data<Addr<AuthManager<A>>>, connnection_info: Result<Query<ConnectionInfo>, actix_web::Error>) -> Result<actix_web::HttpResponse, YummyError> {
    log::debug!("Websocket connection: {:?}", connnection_info);
    let config = config.get_ref();

    let connnection_info = match connnection_info {
        Ok(connnection_info) => connnection_info.into_inner(),
        Err(error) => return Err(YummyError::WebsocketConnectArgument(error.to_string()))
    };

    ws::start(GameWebsocket::new(config.clone(), connnection_info, manager.get_ref().clone()), &req, stream)
        .map_err(|error| YummyError::from(error))
}

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

macro_rules! message_validate {
    ($model: expr) => {
        match $model.validate() {
            Ok(_) => $model,
            Err(e) => return Err(anyhow::anyhow!(e))
        }
    }
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

    fn auth(&self, auth_type: AuthType, ctx: &mut ws::WebsocketContext<Self>) -> anyhow::Result<()> {
        match auth_type {
            AuthType::Email { email, password, if_not_exist_create } => spawn_future!(self.auth.send(message_validate!(EmailAuth { email: email.clone(), password: password.clone(), if_not_exist_create })), self, ctx),
            AuthType::DeviceId { id } => spawn_future!(self.auth.send(message_validate!(DeviceIdAuth::new(id.clone()))), self, ctx),
            AuthType::Refresh { token } => spawn_future!(self.auth.send(RefreshToken(token)), self, ctx),
            _ => ()
        };
        Ok(())
    }

    fn execute_message(&self, message: String, ctx: &mut ws::WebsocketContext<Self>) -> anyhow::Result<()> {
        match serde_json::from_str::<Request>(&message) {
            Ok(message) => {
                match message {
                    Request::Auth { auth_type } => self.auth(auth_type, ctx)
                }
            }
            Err(error) => {
                println!("{:?}", error);
                Err(anyhow::anyhow!("Wrong message format"))
            }
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

        if let Err(error) = result {
            ctx.text(String::from(GenericAnswer::new(false, error.to_string())));
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

#[cfg(test)]
mod tests {
    
    use actix::Actor;
    use actix_test::TestServer;
    use actix_web::web::get;
    use actix_web::HttpResponse;
    use actix_web::error::InternalError;
    use actix_web::web::{QueryConfig, JsonConfig};
    use actix_web::{web::Data, App};
    use database::{create_database, create_connection};
    use general::web::Answer;
    use manager::api::auth::AuthManager;
    use serde_json::json;
    use std::sync::Arc;
    
    use super::*;
    use super::client::*;
    use crate::json_error_handler;

    pub fn create_websocket_server() -> TestServer {
        actix_test::start(move || {
    
            let config = ::general::config::get_configuration();
            let connection = create_connection(":memory:").unwrap();
            create_database(&mut connection.clone().get().unwrap()).unwrap();
            let auth_manager = Data::new(AuthManager::<database::auth::AuthStore>::new(config.clone(), Arc::new(connection)).start());

            let query_cfg = QueryConfig::default()
                .error_handler(|err, _| {
                    log::error!("{:?}", err);
                    InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
                });

            App::new()
                .app_data(auth_manager)
                .app_data(query_cfg)
                .app_data(JsonConfig::default().error_handler(json_error_handler))
                .app_data(Data::new(config))
                .route("/v1/socket/", get().to(websocket_endpoint::<database::auth::AuthStore>))
        })
    }

    #[actix_web::test]
    async fn message_format_validate_1() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket/")).await;

        let request = json!({
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<Answer>(&receive.unwrap())?;
        assert!(!response.status);
        Ok(())
    }

    #[actix_web::test]
    async fn message_format_validate_2() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket/")).await;

        let request = json!({
            "type": "Wrong type"
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<Answer>(&receive.unwrap())?;
        assert!(!response.status);
        Ok(())
    }

    #[actix_web::test]
    async fn message_format_validate_3() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket/")).await;

        let request = json!({
            "wrong type": "Auth"
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<Answer>(&receive.unwrap())?;
        assert!(!response.status);
        Ok(())
    }

    #[actix_web::test]
    async fn message_format_validate_4() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket/")).await;

        let request = json!({
            "Type": ""
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<Answer>(&receive.unwrap())?;
        assert!(!response.status);
        Ok(())
    }

    #[actix_web::test]
    async fn auth_via_device_id() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket/")).await;

        let request = json!({
            "type": "Auth",
            "auth_type": "DeviceId",
            "id": "1234567890"
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<Answer>(&receive.unwrap())?;
        assert!(response.status);
        Ok(())
    }

    #[actix_web::test]
    async fn fail_auth_via_device_id_1() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket/")).await;

        let request = json!({
            "type": "Auth",
            "auth_type": "DeviceId",
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<Answer>(&receive.unwrap())?;
        assert!(!response.status);
        Ok(())
    }

    #[actix_web::test]
    async fn fail_auth_via_device_id_2() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket/")).await;

        let request = json!({
            "type": "Auth",
            "auth_type": "DeviceId",
            "id": ""
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
        assert!(!response.status);
        assert_eq!(response.result.unwrap(), "id: Length should be between 8 to 128 chars".to_string());
        Ok(())
    }

    #[actix_web::test]
    async fn fail_auth_via_device_id_3() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket/")).await;

        let request = json!({
            "type": "Auth",
            "auth_type": "DeviceId",
            "id": 123
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<Answer>(&receive.unwrap())?;
        assert!(!response.status);
        Ok(())
    }

    #[actix_web::test]
    async fn auth_via_email_1() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket/")).await;

        let request = json!({
            "type": "Auth",
            "auth_type": "Email",
            "email": "erhanbaris@gmail.com",
            "password": "erhan",
            "create": true
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
        assert!(response.status);
        assert!(&response.result.is_some());
        Ok(())
    }

    #[actix_web::test]
    async fn auth_via_email_2() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket/")).await;

        let request = json!({
            "type": "Auth",
            "auth_type": "Email",
            "email": "erhanbaris@gmail.com",
            "password": "erhan"
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
        assert!(!response.status);
        assert_eq!(&response.result.unwrap(), "Email and/or password not valid");
        Ok(())
    }

    #[actix_web::test]
    async fn auth_via_email_3() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket/")).await;

        let request = json!({
            "type": "Auth",
            "auth_type": "Email",
            "email": "erhanbaris@gmail.com",
            "password": "erhan",
            "create": false
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
        assert!(!response.status);
        assert_eq!(&response.result.unwrap(), "Email and/or password not valid");
        Ok(())
    }

    #[actix_web::test]
    async fn auth_via_email_4() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket/")).await;

        let request = json!({
            "type": "Auth",
            "auth_type": "Email",
            "email": "",
            "password": ""
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
        assert!(!response.status);

        let error_message = response.result.unwrap();
        assert!(error_message == "password: Length should be between 3 to 32 chars\nemail: Email address is not valid"
             || error_message ==  "email: Email address is not valid\npassword: Length should be between 3 to 32 chars"
        );
        Ok(())
    }

    #[actix_web::test]
    async fn auth_via_email_5() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket/")).await;

        // Not valid
        let request = json!({
            "type": "Auth",
            "auth_type": "Email",
            "email": "erhanbaris@gmail.com",
            "password": "erhan"
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
        assert!(!response.status);

        // Register with right information
        let request = json!({
            "type": "Auth",
            "auth_type": "Email",
            "email": "erhanbaris@gmail.com",
            "password": "erhan",
            "create": true
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
        assert!(response.status);

        let request = json!({
            "type": "Auth",
            "auth_type": "Email",
            "email": "erhanbaris@gmail.com",
            "password": "erhan"
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
        assert!(response.status);

        Ok(())
    }

    #[actix_web::test]
    async fn auth_via_email_6() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket/")).await;

        // Not valid
        let request = json!({
            "type": "Auth",
            "auth_type": "Email",
            "email": "erhanbaris@gmail.com",
            "password": "erhan"
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
        assert!(!response.status);

        // Register with right information
        let request = json!({
            "type": "Auth",
            "auth_type": "Email",
            "email": "erhanbaris@gmail.com",
            "password": "erhan",
            "create": true
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
        assert!(response.status);

        let request = json!({
            "type": "Auth",
            "auth_type": "Email",
            "email": "erhanbaris@gmail.com",
            "password": "erhan"
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
        assert!(response.status);

        Ok(())
    }

    #[actix_web::test]
    async fn fail_token_refresh_1() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket/")).await;

        // Not valid
        let request = json!({
            "type": "Auth",
            "auth_type": "Refresh",
            "token": "erhanbaris"
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
        assert!(!response.status);
        assert_eq!(&response.result.unwrap(), "Token is not valid");

        Ok(())
    }

    #[actix_web::test]
    async fn fail_token_refresh_2() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket/")).await;

        // Not valid
        let request = json!({
            "type": "Auth",
            "auth_type": "Refresh"
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
        assert!(!response.status);
        assert_eq!(&response.result.unwrap(), "Wrong message format");

        Ok(())
    }

    #[actix_web::test]
    async fn token_refresh_1() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket/")).await;

        let request = json!({
            "type": "Auth",
            "auth_type": "DeviceId",
            "id": "1234567890"
        });
        client.send(request).await;

        let receive = client.get_text().await;
        assert!(receive.is_some());

        let token = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?.result.unwrap();

        // Not valid
        let request = json!({
            "type": "Auth",
            "auth_type": "Refresh",
            "token": token
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
        assert!(response.status);

        Ok(())
    }
}