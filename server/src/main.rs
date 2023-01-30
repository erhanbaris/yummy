#![forbid(unsafe_code)]
mod api;

use std::sync::Arc;

use general::config::{get_configuration, configure_environment};
use general::tls::load_rustls_config;
use general::web::json_error_handler;

use manager::plugin::lua::*;
use manager::conn::ConnectionManager;
use manager::user::UserManager;
use manager::auth::AuthManager;

use manager::plugin::PluginBuilder;

use actix::Actor;
use actix_web::error::InternalError;
use actix_web::web::{JsonConfig, QueryConfig};
use actix_web::{web, HttpResponse};
use actix_web::{middleware, App, HttpServer, web::Data};

use crate::api::websocket::websocket_endpoint;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    use general::state::YummyState;
    use manager::room::RoomManager;

    configure_environment();
    let config = get_configuration();
    
    tracing_subscriber::fmt::init();

    let server_bind = format!("{}:{}", config.bind_ip, config.bind_port);
    log::info!("Yummy is starting...");
    log::info!("Binding at   \"{}\"", server_bind);
    log::info!("Server name  \"{}\"", config.server_name);
    log::info!("Log level is \"{}\"", config.rust_log.to_string());

    let database = Arc::new(database::create_connection(&config.database_path).unwrap());
    let mut connection = database.clone().get().unwrap();
    database::create_database(&mut connection).unwrap_or_default();

    #[cfg(feature = "stateless")]
    let redis_client = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();

    let states = YummyState::new(config.clone(), #[cfg(feature = "stateless")] redis_client.clone());

    let mut builder = PluginBuilder::default();
    builder.add_installer(Box::new(LuaPluginInstaller::default()));

    let executer = Arc::new(builder.build(config.clone()));

    let user_manager = Data::new(UserManager::<database::SqliteStore>::new(config.clone(), states.clone(), database.clone(), executer.clone()).start());
    let room_manager = Data::new(RoomManager::<database::SqliteStore>::new(config.clone(), states.clone(), database.clone()).start());
    let conn_manager = Data::new(ConnectionManager::new(config.clone(), states.clone(), executer.clone(), #[cfg(feature = "stateless")] redis_client).start());
    let auth_manager = Data::new(AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), database.clone(), executer.clone()).start());
    
    let data_config = Data::new(config.clone());

    let server = HttpServer::new(move || {
        let query_cfg = QueryConfig::default()
            .error_handler(|err, _| {
                log::error!("{:?}", err);
                InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
            });

        App::new()
            .app_data(query_cfg)
            .app_data(JsonConfig::default().error_handler(json_error_handler))
            .app_data(data_config.clone())
            .app_data(auth_manager.clone())
            .app_data(user_manager.clone())
            .app_data(room_manager.clone())
            .app_data(conn_manager.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .route("/v1/socket", web::get().to(websocket_endpoint::<database::SqliteStore>))
    });

    match load_rustls_config(config.clone()) {
        Some(rustls_config) => server.bind_rustls(server_bind, rustls_config)?.run().await,
        None => server.bind(server_bind)?.run().await
    }
}
