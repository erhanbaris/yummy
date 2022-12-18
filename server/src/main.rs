#![forbid(unsafe_code)]
mod api;

use general::config::{get_configuration, get_env_var};
use general::state::SendMessage;
use manager::api::conn::CommunicationManager;
use manager::api::user::UserManager;
use std::sync::Arc;

use manager::api::auth::AuthManager;

use actix_web::error::InternalError;

use actix::{Actor, Recipient};
use actix_web::error::{JsonPayloadError};
use actix_web::web::{JsonConfig, QueryConfig};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web::{middleware, App, HttpServer, web::Data};

pub fn json_error_handler(err: JsonPayloadError, _: &HttpRequest) -> actix_web::Error {
    let detail = err.to_string();
    let res = HttpResponse::BadRequest().body("error");
    log::error!("Json parse issue: {}", detail);
    
    InternalError::from_response("Json format is not valid. Please check request definition.", res).into()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use general::state::YummyState;
    use manager::api::{room::RoomManager};

    let server_bind = get_env_var("SERVER_BIND", "0.0.0.0:9090".to_string());
    let rust_log_level = get_env_var("RUST_LOG", "debug,backend,actix_web=debug".to_string());
    
    tracing_subscriber::fmt::init();
    std::env::set_var("RUST_LOG", &rust_log_level);
    let config = get_configuration();

    log::info!("Yummy is starting...");
    log::info!("Binding at \"{}\"", server_bind);
    log::info!("Server name \"{}\"", config.server_name);
    log::info!("Log level is \"{}\"", rust_log_level);

    let database = Arc::new(database::create_connection(&config.database_url).unwrap());
    let mut connection = database.clone().get().unwrap();
    database::create_database(&mut connection).unwrap_or_default();

    #[cfg(feature = "stateless")]
    let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();

    let conn_manager = CommunicationManager::new(config.clone()).start();
    let conn_recipient: Recipient<SendMessage> = conn_manager.clone().recipient();
    let states = YummyState::new(config.clone(), #[cfg(feature = "stateless")] conn, #[cfg(feature = "stateless")] conn_recipient);

    let auth_manager = Data::new(AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), database.clone()).start());
    let user_manager = Data::new(UserManager::<database::SqliteStore>::new(config.clone(), states.clone(), database.clone()).start());
    let room_manager = Data::new(RoomManager::<database::SqliteStore>::new(config.clone(), states, database.clone()).start());
        
    HttpServer::new(move || {
        let config = get_configuration();

        let query_cfg = QueryConfig::default()
            .error_handler(|err, _| {
                log::error!("{:?}", err);
                InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
            });

        App::new()
            .app_data(query_cfg)
            .app_data(JsonConfig::default().error_handler(json_error_handler))
            .app_data(Data::new(config))
            .app_data(auth_manager.clone())
            .app_data(user_manager.clone())
            .app_data(room_manager.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            
            //Websocket
            .route("/v1/socket", web::get().to(crate::api::websocket::websocket_endpoint::<database::SqliteStore>))
    })
    .bind(server_bind)?
    .run()
    .await
}
