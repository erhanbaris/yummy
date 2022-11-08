mod api;
mod core;
mod endpoint;
mod websocket;

use ::core::config::{get_configuration, get_env_var};

use actix_web::error::InternalError;

use actix::Actor;
use actix_web::error::{JsonPayloadError};
use actix_web::web::{JsonConfig, QueryConfig};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web::{middleware, App, HttpServer, web::Data};
use manager::GameManager;

use crate::endpoint::websocket_endpoint;

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
    
    std::env::set_var("RUST_LOG", &rust_log_level);
    env_logger::init();

    log::info!("Yummy Starting...");
    log::info!("Binding at \"{}\"", server_bind);
    log::info!("Log level is \"{}\"", rust_log_level);

    HttpServer::new(move || {
        // Read configuration from environment
        let config = get_configuration();

        let query_cfg = QueryConfig::default()
            .error_handler(|err, _| {
                log::error!("{:?}", err);
                InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
            });
        
        let game_manager = Data::new(GameManager::new(config.clone()).start());

        App::new()
        .app_data(query_cfg)
            .app_data(JsonConfig::default().error_handler(json_error_handler))
            .app_data(Data::new(config))
            .app_data(game_manager.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())

            // Apis
            .service(web::scope("/v1").configure(api::account::v1_scoped_config))
            
            //Websocket
            .route("/v1/socket/", web::get().to(websocket_endpoint))

    })
    .bind(server_bind)?
    .run()
    .await
}
