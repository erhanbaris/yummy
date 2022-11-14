mod api;
mod websocket;

use general::config::{get_configuration, get_env_var};
use std::sync::Arc;


use manager::api::auth::AuthManager;
use tracing_subscriber;

use actix_web::error::InternalError;

use actix::Actor;
use actix_web::error::{JsonPayloadError};
use actix_web::web::{JsonConfig, QueryConfig};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web::{middleware, App, HttpServer, web::Data};

use crate::websocket::websocket_endpoint;
use crate::api::http_query;

pub fn json_error_handler(err: JsonPayloadError, _req: &HttpRequest) -> actix_web::Error {
    let detail = err.to_string();
    let res = HttpResponse::BadRequest().body("error");
    log::error!("Json parse issue: {}", detail);

    InternalError::from_response(err, res).into()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server_bind = get_env_var("SERVER_BIND", "0.0.0.0:9090".to_string());
    let rust_log_level = get_env_var("RUST_LOG", "debug,backend,actix_web=debug".to_string());
    let message = crate::websocket::request::Request::Auth {
        auth_type: crate::websocket::request::AuthType::Refresh {
            token: "asd".to_string()
        }
    };

    print!("{:}", serde_json::to_string(&message).unwrap());
    
    tracing_subscriber::fmt::init();
    std::env::set_var("RUST_LOG", &rust_log_level);

    log::info!("Yummy is starting...");
    log::info!("Binding at \"{}\"", server_bind);
    log::info!("Log level is \"{}\"", rust_log_level);

    let config = get_configuration();
    let database = Arc::new(database::create_connection(&config.database_url).unwrap());
    let mut connection = database.clone().get().unwrap();
    database::create_database(&mut connection).unwrap_or_default();
    let auth_manager = Data::new(AuthManager::<database::auth::AuthStore>::new(config.clone(), database.clone()).start());
        
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
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())

            // Account api
            .service(web::scope("/v1/account").configure(api::account::v1_scoped_config::<database::auth::AuthStore>))
            
            //Websocket
            .route("/v1/socket/", web::get().to(websocket_endpoint::<database::auth::AuthStore>))
            .route("/v1/query", web::post().to(http_query::<database::auth::AuthStore>))
            

    })
    .bind(server_bind)?
    .run()
    .await
}
