#![forbid(unsafe_code)]
mod api;

use general::config::{get_configuration, configure_environment};
use manager::api::conn::ConnectionManager;
use manager::api::user::UserManager;
use std::sync::Arc;

use manager::api::auth::AuthManager;

use actix_web::error::InternalError;

use actix::Actor;
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
    let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();

    let states = YummyState::new(config.clone(), #[cfg(feature = "stateless")] conn.clone());

    let user_manager = Data::new(UserManager::<database::SqliteStore>::new(config.clone(), states.clone(), database.clone()).start());
    let room_manager = Data::new(RoomManager::<database::SqliteStore>::new(config.clone(), states.clone(), database.clone()).start());
    let conn_manager = Data::new(ConnectionManager::new(config.clone(), states.clone(), #[cfg(feature = "stateless")] conn).start());
    let auth_manager = Data::new(AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), database.clone()).start());
    
    let config = Data::new(config);

    HttpServer::new(move || {
        let query_cfg = QueryConfig::default()
            .error_handler(|err, _| {
                log::error!("{:?}", err);
                InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
            });

        App::new()
            .app_data(query_cfg)
            .app_data(JsonConfig::default().error_handler(json_error_handler))
            .app_data(config.clone())
            .app_data(auth_manager.clone())
            .app_data(user_manager.clone())
            .app_data(room_manager.clone())
            .app_data(conn_manager.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            
            //Websocket
            .route("/v1/socket", web::get().to(crate::api::websocket::websocket_endpoint::<database::SqliteStore>))
    })
    .bind(server_bind)?
    .run()
    .await
}
