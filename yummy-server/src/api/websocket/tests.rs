
use actix::Actor;
use actix_test::{TestServer, TestServerConfig};
use actix_web::web::get;
use actix_web::HttpResponse;
use actix_web::error::InternalError;
use actix_web::web::{QueryConfig, JsonConfig};
use actix_web::{web::Data, App};
use yummy_cache::state::YummyState;
use yummy_cache::state_resource::ResourceFactory;
use yummy_database::{create_database, create_connection, DefaultDatabaseStore};
use yummy_model::meta::UserMetaAccess;
use yummy_testing::model::{ReceiveError, AuthenticatedModel, RoomCreated};
use yummy_general::tls::load_temporary_rustls_config;
use yummy_model::web::Answer;
use yummy_manager::auth::AuthManager;
use yummy_manager::conn::ConnectionManager;
use yummy_manager::plugin::PluginExecuter;
use serde_json::json;
use uuid::Uuid;
use std::env::temp_dir;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use yummy_model::web::json_error_handler;
use yummy_testing::model::MeModel;

use super::*;

macro_rules! custom_id_auth {
    ($client: expr, $custom_id: expr) => {
        $client.send(json!({
            "type": "AuthCustomId",
            "id": $custom_id
        })).await;
        let auth_receive = $client.get_text().await;
        assert!(auth_receive.is_some());
    };
    ($client: expr) => {
        $client.send(json!({
            "type": "AuthCustomId",
            "id": "1234567890"
        })).await;
        let auth_receive = $client.get_text().await;
        assert!(auth_receive.is_some());
    };
}

macro_rules! get_my_id {
    ($client: expr) => {
        {
            $client.send(json!({
                "type": "Me"
            })).await;
        
            let receive = $client.get_text().await;
            assert!(receive.is_some());
        
            let response = serde_json::from_str::<MeModel>(&receive.unwrap())?;
            assert!(response.status);
        
            response.id.to_string()
        }
    };
}

macro_rules! get_me {
    ($client: expr) => {
        {
            $client.send(json!({
                "type": "Me"
            })).await;
        
            let receive = $client.get_text().await;
            assert!(receive.is_some());
        
            let response = serde_json::from_str::<MeModel>(&receive.unwrap())?;
            assert!(response.status);
        
            response
        }
    };
}

macro_rules! update_meta {
    ($client: expr, $meta: tt) => {
        let request = json!({
            "type": "UpdateUser",
            "metas": $meta
        });

        $client.send(request).await;
        let receive = $client.get_text().await;
        assert!(receive.is_some());
    
        let response = serde_json::from_str::<Answer>(&receive.unwrap())?;
        assert!(response.status);
    };
}

pub fn create_websocket_server(config: Arc<YummyConfig>) -> TestServer {
    create_websocket_server_with_config(config, TestServerConfig::default())
}

pub fn create_websocket_server_with_tls(config: Arc<YummyConfig>) -> TestServer {
    let test_server_config = TestServerConfig::default();
    let test_server_config = test_server_config.rustls(load_temporary_rustls_config(config.clone()).unwrap());
    create_websocket_server_with_config(config, test_server_config)
}

pub fn create_websocket_server_with_config(config: Arc<YummyConfig>, test_server_config: TestServerConfig) -> TestServer {
    let config = config.clone();
    
    actix_test::start_with(test_server_config, move || {
        let mut db_location = temp_dir();
        db_location.push(format!("{}.db", Uuid::new_v4()));

        let connection = create_connection(db_location.to_str().unwrap()).unwrap();
        create_database(&mut connection.clone().get().unwrap()).unwrap();
        
        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();

        let resource_factory = ResourceFactory::<DefaultDatabaseStore>::new(Arc::new(connection.clone()));
        let states = YummyState::new(config.clone(), Box::new(resource_factory), #[cfg(feature = "stateless")] conn.clone());
        let connection = Arc::new(connection);
        let executer = Arc::new(PluginExecuter::new(config.clone(), states.clone(), connection.clone()));

        ConnectionManager::new(config.clone(), states.clone(), executer.clone(), #[cfg(feature = "stateless")] conn.clone()).start();

        let auth_manager = Data::new(AuthManager::<yummy_database::SqliteStore>::new(config.clone(), states.clone(), connection.clone(), executer.clone()).start());
        let user_manager = Data::new(UserManager::<yummy_database::SqliteStore>::new(config.clone(), states.clone(), connection.clone(), executer.clone()).start());
        let room_manager = Data::new(RoomManager::<yummy_database::SqliteStore>::new(config.clone(), states, connection.clone(), executer.clone()).start());

        let query_cfg = QueryConfig::default()
            .error_handler(|err, _| {
                log::error!("{:?}", err);
                InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
            });

        App::new()
            .app_data(auth_manager)
            .app_data(user_manager)
            .app_data(room_manager)
            .app_data(query_cfg)
            .app_data(JsonConfig::default().error_handler(json_error_handler))
            .app_data(Data::new(config.clone()))
            .route("/v1/socket", get().to(websocket_endpoint::<yummy_database::SqliteStore>))
    })
}

#[actix_web::test]
async fn https_test() -> anyhow::Result<()> {
    let server = create_websocket_server_with_tls(yummy_model::config::get_configuration());

    let url = server.url("/v1/socket");
    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(url, yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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
async fn message_format_validate_1() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket"), yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket"), yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    let request = json!({
        "type": "AuthDeviceId",
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
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    let request = json!({
        "type": "AuthDeviceId"
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
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    let request = json!({
        "type": "AuthDeviceId",
        "id": ""
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<ReceiveError>(&receive.unwrap())?;
    assert!(!response.status);
    assert_eq!(response.error, "id: Length should be between 8 to 128 chars".to_string());
    Ok(())
}

#[actix_web::test]
async fn fail_auth_via_device_id_3() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    let request = json!({
        "type": "AuthDeviceId",
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
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    custom_id_auth!(client, "1234567890");
    Ok(())
}

#[actix_web::test]
async fn fail_auth_via_custom_id_1() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    let request = json!({
        "type": "AuthCustomId",
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<ReceiveError>(&receive.unwrap())?;
    assert!(!response.status);
    Ok(())
}

#[actix_web::test]
async fn fail_auth_via_custom_id_2() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    let request = json!({
        "type": "AuthCustomId",
        "id": ""
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<ReceiveError>(&receive.unwrap())?;
    assert!(!response.status);
    assert_eq!(response.error, "id: Length should be between 8 to 128 chars".to_string());
    Ok(())
}

#[actix_web::test]
async fn fail_auth_via_custom_id_3() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    let request = yummy_model::request::Request::Auth {
        request_id: None,
        auth_type: yummy_model::request::RequestAuthType::CustomId {
            id: "123".to_string()
        }
    };

    let request = serde_json::to_string(&request).unwrap();
    println!("request {:?}", request);

    let request = json!({
        "type": "AuthCustomId",
        "id": "123"
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    println!("receive {:?}", &receive);
    let response = serde_json::from_str::<Answer>(&receive.unwrap())?;
    assert!(!response.status);
    Ok(())
}

#[actix_web::test]
async fn auth_via_email_1() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    let request = json!({
        "type": "AuthEmail",
        "email": "erhanbaris@gmail.com",
        "password": "erhan",
        "create": true
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<AuthenticatedModel>(&receive.unwrap())?;
    assert!(response.status);
    Ok(())
}

#[actix_web::test]
async fn auth_via_email_2() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    let request = json!({
        "type": "AuthEmail",
        "email": "erhanbaris@gmail.com",
        "password": "erhan"
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<ReceiveError>(&receive.unwrap())?;
    assert!(!response.status);
    assert_eq!(&response.error, "Email and/or password not valid");
    Ok(())
}

#[actix_web::test]
async fn auth_via_email_3() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    let request = json!({
        "type": "AuthEmail",
        "email": "erhanbaris@gmail.com",
        "password": "erhan",
        "create": false
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<ReceiveError>(&receive.unwrap())?;
    assert!(!response.status);
    assert_eq!(&response.error, "Email and/or password not valid");
    Ok(())
}

#[actix_web::test]
async fn auth_via_email_4() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    let request = json!({
        "type": "AuthEmail",
        "email": "",
        "password": ""
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<ReceiveError>(&receive.unwrap())?;
    assert!(!response.status);

    let error_message = response.error;
    assert!(error_message == "password: Length should be between 3 to 32 chars\nemail: Email address is not valid"
            || error_message ==  "email: Email address is not valid\npassword: Length should be between 3 to 32 chars"
    );
    Ok(())
}

#[actix_web::test]
async fn auth_via_email_5() -> anyhow::Result<()> {
    let mut config = yummy_model::config::get_configuration().deref().clone();
    config.connection_restore_wait_timeout = Duration::from_secs(1);
    config.heartbeat_interval = Duration::from_secs(1);
    config.heartbeat_timeout = Duration::from_secs(1);

    let server = create_websocket_server(Arc::new(config));

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    // Not valid
    let request = json!({
        "type": "AuthEmail",
        "email": "erhanbaris@gmail.com",
        "password": "erhan"
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<ReceiveError>(&receive.unwrap())?;
    assert!(!response.status);

    // Register with right information
    let request = json!({
        "type": "AuthEmail",
        "email": "erhanbaris@gmail.com",
        "password": "erhan",
        "create": true
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<AuthenticatedModel>(&receive.unwrap())?;
    assert!(response.status);

    client.disconnect().await;
    actix::clock::sleep(std::time::Duration::new(3, 0)).await;

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;
    
    // Login again
    let request = json!({
        "type": "AuthEmail",
        "email": "erhanbaris@gmail.com",
        "password": "erhan"
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<AuthenticatedModel>(&receive.unwrap())?;
    assert!(response.status);

    Ok(())
}

#[actix_web::test]
async fn auth_via_email_6() -> anyhow::Result<()> {
    let mut config = yummy_model::config::get_configuration().deref().clone();
    config.connection_restore_wait_timeout = Duration::from_secs(1);
    config.heartbeat_interval = Duration::from_secs(1);
    config.heartbeat_timeout = Duration::from_secs(1);

    let server = create_websocket_server(Arc::new(config));

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    // Not valid
    let request = json!({
        "type": "AuthEmail",
        "email": "erhanbaris@gmail.com",
        "password": "erhan"
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<ReceiveError>(&receive.unwrap())?;
    assert!(!response.status);

    // Register with right information
    let request = json!({
        "type": "AuthEmail",
        "email": "erhanbaris@gmail.com",
        "password": "erhan",
        "create": true
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<AuthenticatedModel>(&receive.unwrap())?;
    assert!(response.status);

    client.disconnect().await;
    actix::clock::sleep(std::time::Duration::new(3, 0)).await;

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    let request = json!({
        "type": "AuthEmail",
        "email": "erhanbaris@gmail.com",
        "password": "erhan"
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<AuthenticatedModel>(&receive.unwrap())?;
    assert!(response.status);

    Ok(())
}

#[actix_web::test]
async fn success_logout() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    // Register with right information
    let request = json!({
        "type": "AuthEmail",
        "email": "erhanbaris@gmail.com",
        "password": "erhan",
        "create": true
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<AuthenticatedModel>(&receive.unwrap())?;
    assert!(response.status);

    // Logout
    let request = json!({
        "type": "Logout"
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<Answer>(&receive.unwrap())?;
    assert!(response.status);

    client.disconnect().await;

    Ok(())
}


#[actix_web::test]
async fn fail_token_refresh_1() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    // Not valid
    let request = json!({
        "type": "RefreshToken",
        "token": "erhanbaris"
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<ReceiveError>(&receive.unwrap())?;
    assert!(!response.status);
    assert_eq!(&response.error, "token: Length should be between 275 to 1024 chars");

    Ok(())
}

#[actix_web::test]
async fn fail_token_refresh_2() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    // Not valid
    let request = json!({
        "type": "RefreshToken",
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<ReceiveError>(&receive.unwrap())?;
    assert!(!response.status);
    assert_eq!(&response.error, "Wrong message format");

    Ok(())
}

#[actix_web::test]
async fn token_refresh_1() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    let request = json!({
        "type": "AuthDeviceId",
        "id": "1234567890"
    });
    client.send(request).await;

    let receive = client.get_text().await;
    assert!(receive.is_some());

    let token = serde_json::from_str::<AuthenticatedModel>(&receive.unwrap())?.token;

    // Not valid
    let request = json!({
        "type": "RefreshToken",
        "token": token
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<AuthenticatedModel>(&receive.unwrap())?;
    assert!(response.status);

    Ok(())
}

#[actix_web::test]
async fn token_restore_1() -> anyhow::Result<()> {
    let mut config = yummy_model::config::get_configuration().deref().clone();
    config.connection_restore_wait_timeout = Duration::from_secs(60);
    config.token_lifetime = Duration::from_secs(60);

    let server = create_websocket_server(Arc::new(config));

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    let request = json!({
        "type": "AuthDeviceId",
        "id": "1234567890"
    });
    client.send(request).await;

    let receive = client.get_text().await;
    assert!(receive.is_some());

    let token = serde_json::from_str::<AuthenticatedModel>(&receive.unwrap())?.token;

    client.disconnect().await;
    
    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    let request = json!({
        "type": "RestoreToken",
        "token": token
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<AuthenticatedModel>(&receive.unwrap())?;
    assert!(response.status);

    Ok(())
}

#[actix_web::test]
async fn fail_token_restore_1() -> anyhow::Result<()> {
    let mut config = yummy_model::config::get_configuration().deref().clone();
    config.token_lifetime = Duration::from_secs(1);

    let server = create_websocket_server(Arc::new(config));

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    let request = json!({
        "type": "AuthDeviceId",
        "id": "1234567890"
    });
    client.send(request).await;

    let receive = client.get_text().await;
    assert!(receive.is_some());

    let token = serde_json::from_str::<AuthenticatedModel>(&receive.unwrap())?.token;

    client.disconnect().await;

    // Wait 3 seconds
    actix::clock::sleep(std::time::Duration::new(3, 0)).await;
    
    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    // Not valid
    let request = json!({
        "type": "RestoreToken",
        "token": token
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<ReceiveError>(&receive.unwrap())?;
    assert_eq!(response.status, false);

    Ok(())
}

/* User test cases */

#[actix_web::test]
async fn user_me_1() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    custom_id_auth!(client, "1234567890");
    get_my_id!(client);

    Ok(())
}

#[actix_web::test]
async fn user_get_1() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    custom_id_auth!(client, "1234567890");

    let id = get_my_id!(client);
    
    client.send(json!({
        "type": "GetUser",
        "user_id": id
    })).await;

    let receive = client.get_text().await;
    assert!(receive.is_some());
    let response = serde_json::from_str::<Answer>(&receive.unwrap())?;
    assert!(response.status);
    
    Ok(())
}


#[actix_web::test]
async fn user_online_status_change() -> anyhow::Result<()> {
    let mut config = yummy_model::config::get_configuration().deref().clone();
    config.token_lifetime = Duration::from_secs(1);
    config.connection_restore_wait_timeout = Duration::from_secs(1);

    let server = create_websocket_server(Arc::new(config));
    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    custom_id_auth!(client, "test user 1");
    let id = get_my_id!(client);

    client.send(json!({
        "type": "GetUser",
        "user_id": id
    })).await;

    let receive = client.get_text().await;
    assert!(receive.is_some());
    let response = serde_json::from_str::<MeModel>(&receive.unwrap())?;
    assert!(response.status);

    assert!(response.online);
    client.disconnect().await;

    actix::clock::sleep(std::time::Duration::new(3, 0)).await;

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;
    custom_id_auth!(client, "test user 2");

    client.send(json!({
        "type": "GetUser",
        "user_id": id
    })).await;

    let receive = client.get_text().await;
    assert!(receive.is_some());
    let response = serde_json::from_str::<MeModel>(&receive.unwrap())?;
    assert!(response.status);

    // Connection timeout
    assert!(!response.online);
    
    Ok(())
}

#[actix_web::test]
async fn user_update_1() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    custom_id_auth!(client);

    client.send(json!({
        "type": "UpdateUser",
        "name": "Erhan BARIS",
        "custom_id": "1234567890",
        "device_id": "987654321",
        "email": "erhanbaris@gmail.com"
    })).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<Answer>(&receive.unwrap())?;
    assert!(response.status);

    client.send(json!({
        "type": "Me"
    })).await;

    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<MeModel>(&receive.unwrap())?;
    assert!(response.status);
    let response = response;

    assert_eq!(response.custom_id.unwrap().as_str(), "1234567890");
    assert_eq!(response.device_id.unwrap().as_str(), "987654321");
    assert_eq!(response.name.unwrap().as_str(), "Erhan BARIS");
    assert_eq!(response.email.unwrap().as_str(), "erhanbaris@gmail.com");

    Ok(())
}

#[actix_web::test]
async fn user_update_2() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    custom_id_auth!(client, "1234567890");

    client.send(json!({
        "type": "UpdateUser",
        "metas": {}
    })).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<Answer>(&receive.unwrap())?;
    assert!(!response.status);

    Ok(())
}

#[actix_web::test]
async fn user_update_3() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    custom_id_auth!(client, "1234567890");
    update_meta!(client, {
        "lat": 3.11133,
        "lon": 5.444,
        "gender": {
            "access": 3,
            "value": "Male"
        }
    });

    Ok(())
}

#[actix_web::test]
async fn user_update_4() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;
    custom_id_auth!(client, "1234567890");
    update_meta!(client, {
        "lat": 3.11133,
        "lon": 5.444,
        "me type": {
            "access": UserMetaAccess::Me as u32,
            "value": 9
        },
        "user type": {
            "access": UserMetaAccess::User as u32,
            "value": 8
        }
    });

    let me = get_me!(client);

    let me = me.metas;
    assert_eq!(me.get("me type"), Some(&serde_json::Value::Number(serde_json::Number::from_f64(9.0).unwrap())));
    assert_eq!(me.get("user type"), Some(&serde_json::Value::Number(serde_json::Number::from_f64(8.0).unwrap())));
    
    Ok(())
}

// Room test cases
#[actix_web::test]
async fn create_room() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    custom_id_auth!(client, "1234567890");

    client.send(json!({
        "type": "CreateRoom"
    })).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<GenericAnswer<RoomCreated>>(&receive.unwrap())?;
    assert!(response.status);
    Ok(())
}

#[actix_web::test]
async fn join_room() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client_1 = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;
    custom_id_auth!(client_1, "client_1");

    let mut client_2 = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;
    custom_id_auth!(client_2, "client_2");

    // Error
    client_1.send(json!({
        "type": "CreateRoom"
    })).await;
    let receive = client_1.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<GenericAnswer<RoomCreated>>(&receive.unwrap())?;
    assert!(response.status);

    let room_id = response.result.room_id;
    
    client_2.send(json!({
        "type": "JoinToRoom",
        "room": room_id,
        "room_user_type": 1
    })).await;

    let receive = client_2.get_text().await;
    assert!(receive.is_some());

    Ok(())
}

#[actix_web::test]
async fn ping_pong() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client_1 = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;
    client_1.ping().await;
    client_1.get_pong().await;

    client_1.ping().await;
    assert_eq!(client_1.get_text().await.unwrap(), String::new());

    Ok(())
}

#[actix_web::test]
async fn pong_ping() -> anyhow::Result<()> {
    let server = create_websocket_server(yummy_model::config::get_configuration());

    let mut client_1 = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;
    client_1.pong().await;
    client_1.get_ping().await;

    client_1.pong().await;
    assert_eq!(client_1.get_text().await.unwrap(), String::new());

    Ok(())
}

#[cfg(feature = "stateless")]
#[actix_web::test]
async fn pub_sub_test() -> anyhow::Result<()> {
    let server_1 = create_websocket_server_with_tls(yummy_model::config::get_configuration());
    let server_2 = create_websocket_server_with_tls(yummy_model::config::get_configuration());

    let mut client_1 = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server_1.url("/v1/socket"), yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;
    let mut client_2 = yummy_general::websocket::WebsocketTestClient::<String, String>::new(server_2.url("/v1/socket"), yummy_model::config::DEFAULT_API_KEY_NAME.to_string(), yummy_model::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    client_1.send(json!({
        "type": "AuthEmail",
        "email": "user1@gmail.com",
        "password": "erhan",
        "create": true
    })).await;

    client_2.send(json!({
        "type": "AuthEmail",
        "email": "user2@gmail.com",
        "password": "erhan",
        "create": true
    })).await;

    let receive_1 = serde_json::from_str::<Answer>(&client_1.get_text().await.unwrap())?;
    let receive_2 = serde_json::from_str::<Answer>(&client_2.get_text().await.unwrap())?;

    assert!(receive_1.status);
    assert!(receive_2.status);

    client_1.send(json!({
        "type": "CreateRoom"
    })).await;
    let receive_1 = serde_json::from_str::<GenericAnswer<RoomCreated>>(&client_1.get_text().await.unwrap())?;
    assert!(receive_1.status);

    let room_id = receive_1.result.room;
    
    client_2.send(json!({
        "type": "JoinToRoom",
        "room": room_id,
        "room_user_type": 1
    })).await;

    let receive = serde_json::from_str::<Joined>(&client_2.get_text().await.unwrap())?;
    assert_eq!(&receive.class_type, "JoinToRoom");
    
    Ok(())
}