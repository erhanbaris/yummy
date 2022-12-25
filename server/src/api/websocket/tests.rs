
use actix::Actor;
use actix_test::{TestServer, TestServerConfig};
use actix_web::web::get;
use actix_web::HttpResponse;
use actix_web::error::InternalError;
use actix_web::web::{QueryConfig, JsonConfig};
use actix_web::{web::Data, App};
use database::model::UserInformationModel;
use database::{create_database, create_connection, RowId};
use general::meta::MetaAccess;
use general::model::UserType;
use general::state::YummyState;
use general::tls::load_temporary_rustls_config;
use general::web::Answer;
use manager::api::auth::AuthManager;
use manager::api::conn::ConnectionManager;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use std::collections::HashMap;
use std::env::temp_dir;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use super::*;
use super::client::*;
use crate::json_error_handler;

#[cfg(feature = "stateless")]
use general::test::cleanup_redis;

#[derive(Default, Clone, Debug, Deserialize)]
pub struct UserInformationResponse {
    pub id: RowId,
    pub name: Option<String>,
    pub email: Option<String>,
    pub device_id: Option<String>,
    pub custom_id: Option<String>,
    pub meta: Option<HashMap<String, serde_json::Value>>,
    pub user_type: UserType,
    pub online: bool,
    pub insert_date: i32,
    pub last_login_date: i32,
}


macro_rules! custom_id_auth {
    ($client: expr, $custom_id: expr) => {
        $client.send(json!({
            "type": "Auth",
            "auth_type": "CustomId",
            "id": $custom_id
        })).await;
        let auth_receive = $client.get_text().await;
        assert!(auth_receive.is_some());
    };
    ($client: expr) => {
        $client.send(json!({
            "type": "Auth",
            "auth_type": "CustomId",
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
                "type": "User",
                "user_type": "Me"
            })).await;
        
            let receive = $client.get_text().await;
            assert!(receive.is_some());
        
            let response = serde_json::from_str::<GenericAnswer<UserInformationModel>>(&receive.unwrap())?;
            assert!(response.status);
        
            response.result.unwrap().id.to_string()
        }
    };
}

macro_rules! get_me {
    ($client: expr) => {
        {
            $client.send(json!({
                "type": "User",
                "user_type": "Me"
            })).await;
        
            let receive = $client.get_text().await;
            assert!(receive.is_some());
        
            let response = serde_json::from_str::<GenericAnswer<UserInformationResponse>>(&receive.unwrap())?;
            assert!(response.status);
        
            response.result.unwrap()
        }
    };
}

macro_rules! update_meta {
    ($client: expr, $meta: tt) => {
        let request = json!({
            "type": "User",
            "user_type": "Update",
            "meta": $meta
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

        #[cfg(feature = "stateless")]
        cleanup_redis(conn.clone());
        let states = YummyState::new(config.clone(), #[cfg(feature = "stateless")] conn.clone());

        ConnectionManager::new(config.clone(), states.clone(), #[cfg(feature = "stateless")] conn.clone()).start();

        let auth_manager = Data::new(AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone())).start());
        let user_manager = Data::new(UserManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone())).start());
        let room_manager = Data::new(RoomManager::<database::SqliteStore>::new(config.clone(), states, Arc::new(connection)).start());

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
            .route("/v1/socket", get().to(websocket_endpoint::<database::SqliteStore>))
    })
}

#[actix_web::test]
async fn https_test() -> anyhow::Result<()> {
    let server = create_websocket_server_with_tls(::general::config::get_configuration());

    let url = server.url("/v1/socket");
    let mut client = WebsocketTestClient::<String, String>::new(url, general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

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
    let server = create_websocket_server(::general::config::get_configuration());

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
    let server = create_websocket_server(::general::config::get_configuration());

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
    let server = create_websocket_server(::general::config::get_configuration());

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
    let server = create_websocket_server(::general::config::get_configuration());

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
    let server = create_websocket_server(::general::config::get_configuration());

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
    let server = create_websocket_server(::general::config::get_configuration());

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
    let server = create_websocket_server(::general::config::get_configuration());

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
    let server = create_websocket_server(::general::config::get_configuration());

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
    let server = create_websocket_server(::general::config::get_configuration());

    let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    custom_id_auth!(client, "1234567890");
    Ok(())
}

#[actix_web::test]
async fn fail_auth_via_custom_id_1() -> anyhow::Result<()> {
    let server = create_websocket_server(::general::config::get_configuration());

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
    let server = create_websocket_server(::general::config::get_configuration());

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
    assert_eq!(response.result.unwrap(), "id: Length should be between 8 to 128 chars".to_string());
    Ok(())
}

#[actix_web::test]
async fn fail_auth_via_custom_id_3() -> anyhow::Result<()> {
    let server = create_websocket_server(::general::config::get_configuration());

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
    let server = create_websocket_server(::general::config::get_configuration());

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
    let server = create_websocket_server(::general::config::get_configuration());

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
    let server = create_websocket_server(::general::config::get_configuration());

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
    let server = create_websocket_server(::general::config::get_configuration());

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
    let mut config = ::general::config::get_configuration().deref().clone();
    config.connection_restore_wait_timeout = Duration::from_secs(1);
    config.heartbeat_interval = Duration::from_secs(1);
    config.heartbeat_timeout = Duration::from_secs(1);

    let server = create_websocket_server(Arc::new(config));

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

    client.disconnect().await;
    actix::clock::sleep(std::time::Duration::new(3, 0)).await;

    let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;
    
    // Login again
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
    let mut config = ::general::config::get_configuration().deref().clone();
    config.connection_restore_wait_timeout = Duration::from_secs(1);
    config.heartbeat_interval = Duration::from_secs(1);
    config.heartbeat_timeout = Duration::from_secs(1);

    let server = create_websocket_server(Arc::new(config));

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

    client.disconnect().await;
    actix::clock::sleep(std::time::Duration::new(3, 0)).await;

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
    assert!(response.status);

    Ok(())
}

#[actix_web::test]
async fn fail_token_refresh_1() -> anyhow::Result<()> {
    let server = create_websocket_server(::general::config::get_configuration());

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
    let server = create_websocket_server(::general::config::get_configuration());

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
    let server = create_websocket_server(::general::config::get_configuration());

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

#[actix_web::test]
async fn token_restore_1() -> anyhow::Result<()> {
    let mut config = ::general::config::get_configuration().deref().clone();
    config.connection_restore_wait_timeout = Duration::from_secs(60);
    config.token_lifetime = Duration::from_secs(60);

    let server = create_websocket_server(Arc::new(config));

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

    client.disconnect().await;
    
    let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    let request = json!({
        "type": "Auth",
        "auth_type": "Restore",
        "token": token
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
    assert!(response.status);

    Ok(())
}

#[actix_web::test]
async fn fail_token_restore_1() -> anyhow::Result<()> {
    let mut config = ::general::config::get_configuration().deref().clone();
    config.token_lifetime = Duration::from_secs(1);

    let server = create_websocket_server(Arc::new(config));

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

    client.disconnect().await;

    // Wait 3 seconds
    actix::clock::sleep(std::time::Duration::new(3, 0)).await;
    
    let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    // Not valid
    let request = json!({
        "type": "Auth",
        "auth_type": "Restore",
        "token": token
    });
    client.send(request).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
    assert_eq!(response.status, false);

    Ok(())
}

/* User test cases */

#[actix_web::test]
async fn user_me_1() -> anyhow::Result<()> {
    let server = create_websocket_server(::general::config::get_configuration());

    let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    custom_id_auth!(client, "1234567890");
    get_my_id!(client);

    Ok(())
}

#[actix_web::test]
async fn user_get_1() -> anyhow::Result<()> {
    let server = create_websocket_server(::general::config::get_configuration());

    let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    custom_id_auth!(client, "1234567890");

    let id = get_my_id!(client);
    
    client.send(json!({
        "type": "User",
        "user_type": "Get",
        "user": id
    })).await;

    let receive = client.get_text().await;
    assert!(receive.is_some());
    let response = serde_json::from_str::<Answer>(&receive.unwrap())?;
    assert!(response.status);
    
    Ok(())
}


#[actix_web::test]
async fn user_online_status_change() -> anyhow::Result<()> {
    let mut config = ::general::config::get_configuration().deref().clone();
    config.token_lifetime = Duration::from_secs(1);
    config.connection_restore_wait_timeout = Duration::from_secs(1);

    let server = create_websocket_server(Arc::new(config));
    let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    custom_id_auth!(client, "test user 1");
    let id = get_my_id!(client);

    client.send(json!({
        "type": "User",
        "user_type": "Get",
        "user": id
    })).await;

    let receive = client.get_text().await;
    assert!(receive.is_some());
    let response = serde_json::from_str::<GenericAnswer<UserInformationModel>>(&receive.unwrap())?;
    assert!(response.status);

    let response = response.result.unwrap();
    assert!(response.online);
    client.disconnect().await;

    actix::clock::sleep(std::time::Duration::new(3, 0)).await;

    let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;
    custom_id_auth!(client, "test user 2");

    client.send(json!({
        "type": "User",
        "user_type": "Get",
        "user": id
    })).await;

    let receive = client.get_text().await;
    assert!(receive.is_some());
    let response = serde_json::from_str::<GenericAnswer<UserInformationModel>>(&receive.unwrap())?;
    assert!(response.status);

    // Connection timeout
    let response = response.result.unwrap();
    assert!(!response.online);
    
    Ok(())
}

#[actix_web::test]
async fn user_update_1() -> anyhow::Result<()> {
    let server = create_websocket_server(::general::config::get_configuration());

    let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    custom_id_auth!(client);

    client.send(json!({
        "type": "User",
        "user_type": "Update",
        "name": "Erhan BARIS",
        "custom_id": "1234567890",
        "device_id": "987654321",
        "email": "erhanbaris@gmail.com"
    })).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
    assert!(response.status);

    client.send(json!({
        "type": "User",
        "user_type": "Me"
    })).await;

    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<GenericAnswer<UserInformationModel>>(&receive.unwrap())?;
    assert!(response.status);
    let response = response.result.unwrap();

    assert_eq!(response.custom_id.unwrap().as_str(), "1234567890");
    assert_eq!(response.device_id.unwrap().as_str(), "987654321");
    assert_eq!(response.name.unwrap().as_str(), "Erhan BARIS");
    assert_eq!(response.email.unwrap().as_str(), "erhanbaris@gmail.com");

    Ok(())
}

#[actix_web::test]
async fn user_update_2() -> anyhow::Result<()> {
    let server = create_websocket_server(::general::config::get_configuration());

    let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    custom_id_auth!(client, "1234567890");

    client.send(json!({
        "type": "User",
        "user_type": "Update",
        "meta": {}
    })).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<Answer>(&receive.unwrap())?;
    assert!(!response.status);

    Ok(())
}

#[actix_web::test]
async fn user_update_3() -> anyhow::Result<()> {
    let server = create_websocket_server(::general::config::get_configuration());

    let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    custom_id_auth!(client, "1234567890");
    update_meta!(client, {
        "lat": 3.11133,
        "lon": 5.444,
        "gender": {
            "access": 4,
            "value": "Male"
        }
    });

    Ok(())
}

#[actix_web::test]
async fn user_update_4() -> anyhow::Result<()> {
    let server = create_websocket_server(::general::config::get_configuration());

    let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;
    custom_id_auth!(client, "1234567890");
    update_meta!(client, {
        "lat": 3.11133,
        "lon": 5.444,
        "admin type": {
            "access": MetaAccess::Admin as u32,
            "value": 10
        },
        "me type": {
            "access": MetaAccess::Me as u32,
            "value": 9
        },
        "user type": {
            "access": MetaAccess::User as u32,
            "value": 8
        }
    });

    let me = get_me!(client);
    assert!(me.meta.is_some());

    let me = me.meta.unwrap();
    assert!(me.get("admin type").is_none());
    assert_eq!(me.get("me type"), Some(&serde_json::Value::Number(serde_json::Number::from_f64(9.0).unwrap())));
    assert_eq!(me.get("user type"), Some(&serde_json::Value::Number(serde_json::Number::from_f64(8.0).unwrap())));
    
    Ok(())
}

// Room test cases


#[actix_web::test]
async fn create_room() -> anyhow::Result<()> {
    let server = create_websocket_server(::general::config::get_configuration());

    let mut client = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;

    custom_id_auth!(client, "1234567890");

    // Error
    client.send(json!({
        "type": "Room",
        "room_type": "Create"
    })).await;
    let receive = client.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
    assert!(response.status);
    assert!(uuid::Uuid::from_str(&response.result.unwrap()).is_ok());
    Ok(())
}

#[actix_web::test]
async fn join_room() -> anyhow::Result<()> {
    let server = create_websocket_server(::general::config::get_configuration());

    let mut client_1 = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;
    custom_id_auth!(client_1, "client_1");

    let mut client_2 = WebsocketTestClient::<String, String>::new(server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;
    custom_id_auth!(client_2, "client_2");

    // Error
    client_1.send(json!({
        "type": "Room",
        "room_type": "Create"
    })).await;
    let receive = client_1.get_text().await;
    assert!(receive.is_some());

    let response = serde_json::from_str::<GenericAnswer<String>>(&receive.unwrap())?;
    assert!(response.status);

    let room_id = response.result.unwrap();
    assert!(uuid::Uuid::from_str(&room_id).is_ok());

    client_2.send(json!({
        "type": "Room",
        "room_type": "Join",
        "room": room_id,
        "room_user_type": "User"
    })).await;

    let receive = client_2.get_text().await;
    assert!(receive.is_some());

    Ok(())
}