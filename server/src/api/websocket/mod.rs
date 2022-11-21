#[cfg(test)]
mod client;

use actix_web::HttpRequest;
use actix_web::web::Data;
use actix_web::web::Payload;
use database::DatabaseTrait;
use general::auth::ApiIntegration;
use general::auth::UserAuth;
use general::error::YummyError;
use general::model::WebsocketMessage;
use general::web::GenericAnswer;
use manager::api::user::UserManager;
use manager::response::Response;
use std::sync::Arc;
use std::time::Instant;


use actix::Actor;
use actix::AsyncContext;
use actix::ActorFutureExt;
use actix::Handler;
use actix::WrapFuture;
use actix::{ActorContext, Addr, Running, StreamHandler, fut};
use actix_web::Result;
use actix_web_actors::ws;
use manager::api::auth::AuthManager;

use general::config::YummyConfig;
use crate::api::process_auth;
use crate::api::process_user;
use crate::api::request::*;

pub async fn websocket_endpoint<DB: DatabaseTrait + Unpin + 'static>(req: HttpRequest, stream: Payload, config: Data<Arc<YummyConfig>>, auth_manager: Data<Addr<AuthManager<DB>>>, user_manager: Data<Addr<UserManager<DB>>>, _: ApiIntegration) -> Result<actix_web::HttpResponse, YummyError> {
    let config = config.get_ref();

    ws::start(GameWebsocket::new(config.clone(), auth_manager.get_ref().clone(), user_manager.get_ref().clone()), &req, stream)
        .map_err(YummyError::from)
}

pub struct GameWebsocket<DB: DatabaseTrait + ?Sized + Unpin + 'static> {
    auth: Addr<AuthManager<DB>>,
    user: Addr<UserManager<DB>>,
    hb: Instant,
    user_auth: Arc<Option<UserAuth>>,
    config: Arc<YummyConfig>,
}

impl<DB: DatabaseTrait + ?Sized + Unpin + 'static> GameWebsocket<DB> {
    pub fn new(
        config: Arc<YummyConfig>,
        auth: Addr<AuthManager<DB>>,
        user: Addr<UserManager<DB>>,
    ) -> Self {
        Self {
            hb: Instant::now(),
            auth,
            user,
            config,
            user_auth: Arc::new(None)
        }
    }

    fn execute_message(&mut self, message: String, ctx: &mut ws::WebsocketContext<Self>) -> anyhow::Result<()> {
        let message = match serde_json::from_str::<Request>(&message) {
            Ok(message) => message,
            Err(_) => return Err(anyhow::anyhow!("Wrong message format"))
        };

        let auth_manager = self.auth.clone();
        let user_manager = self.user.clone();
        let user_info = self.user_auth.clone();

        let future = Box::pin(async {
            let result = match message {
                Request::Auth { auth_type } => process_auth(auth_type, auth_manager).await,
                Request::User { user_type } => process_user(user_type, user_manager, user_info).await
            };
            result
        });

        let actor_future = future
            .into_actor(self)
            .then(move |result, actor, ctx| {
                match result {
                    Ok(response) => match response {
                        Response::Auth(token, auth) => {
                            actor.user_auth = Arc::new(Some(UserAuth {
                                user: auth.id,
                                session: auth.session
                            }));

                            ctx.text(serde_json::to_string(&GenericAnswer::success(token)).unwrap_or_default())
                        },
                        Response::UserPrivateInfo(model) => ctx.text(serde_json::to_string(&GenericAnswer::success(model)).unwrap_or_default()),
                        Response::UserPublicInfo(model) => ctx.text(serde_json::to_string(&GenericAnswer::success(model)).unwrap_or_default()),
                        Response::None => ()
                    },
                    Err(error) => {
                        tracing::error!("{:?}", error);
                        ctx.text(serde_json::to_string(&GenericAnswer::fail(error.to_string())).unwrap_or_default())
                    }
                }
                fut::ready(())
            });

        // Spawns a future into the context.
        ctx.spawn(actor_future);
        Ok(())
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

impl<DB: DatabaseTrait + ?Sized + Unpin + 'static> Actor for GameWebsocket<DB> {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::debug!("New socket started");
        self.hb(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        Running::Stop
    }
}

impl<DB: DatabaseTrait + ?Sized + Unpin + 'static> StreamHandler<Result<ws::Message, ws::ProtocolError>>
    for GameWebsocket<DB>
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

impl<DB: DatabaseTrait + ?Sized + Unpin + 'static> Handler<WebsocketMessage> for GameWebsocket<DB> {
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
            let auth_manager = Data::new(AuthManager::<database::SqliteStore>::new(config.clone(), Arc::new(connection.clone())).start());
            let user_manager = Data::new(UserManager::<database::SqliteStore>::new(Arc::new(connection)).start());

            let query_cfg = QueryConfig::default()
                .error_handler(|err, _| {
                    log::error!("{:?}", err);
                    InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
                });

            App::new()
            .app_data(auth_manager)
            .app_data(user_manager)
            .app_data(query_cfg)
                .app_data(JsonConfig::default().error_handler(json_error_handler))
                .app_data(Data::new(config))
                .route("/v1/socket", get().to(websocket_endpoint::<database::SqliteStore>))
        })
    }

    #[actix_web::test]
    async fn message_format_validate_1() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket"), general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket"), general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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
    async fn auth_via_custom_id() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

        let request = json!({
            "type": "Auth",
            "auth_type": "CustomId",
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
    async fn fail_auth_via_custom_id_1() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

        let request = json!({
            "type": "Auth",
            "auth_type": "CustomId",
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<Answer>(&receive.unwrap())?;
        assert!(!response.status);
        Ok(())
    }

    #[actix_web::test]
    async fn fail_auth_via_custom_id_2() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

        let request = json!({
            "type": "Auth",
            "auth_type": "CustomId",
            "id": ""
        });
        client.send(request).await;
        let receive = client.get_text().await;
        assert!(receive.is_some());

        let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
        assert!(!response.status);
        assert_eq!(response.result.unwrap(), "id: Length should be between 3 to 128 chars".to_string());
        Ok(())
    }

    #[actix_web::test]
    async fn fail_auth_via_custom_id_3() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

        let request = json!({
            "type": "Auth",
            "auth_type": "CustomId",
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

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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
        assert_eq!(&response.result.unwrap(), "token: Length should be between 275 to 1024 chars");

        Ok(())
    }

    #[actix_web::test]
    async fn fail_token_refresh_2() -> anyhow::Result<()> {
        let server = create_websocket_server();

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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

        let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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