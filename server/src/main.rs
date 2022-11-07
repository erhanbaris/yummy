//mod websocket;
mod auth;
mod game;
mod model;
mod config;
mod manager;
mod websocket;

use actix::Actor;
use actix_web::web;
use actix_web::{middleware, App, HttpServer, web::Data};

use crate::config::*;
use crate::websocket::websocket_endpoint;
use crate::game::GameManager;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server_bind = get_env_var("SERVER_BIND", "0.0.0.0:9090".to_string());
    let rust_log_level = get_env_var("RUST_LOG", "debug,backend,actix_web=info".to_string());
    
    std::env::set_var("RUST_LOG", &rust_log_level);
    env_logger::init();

    log::info!("Yummy Starting...");
    log::info!("Binding at \"{}\"", server_bind);
    log::info!("Log level is \"{}\"", rust_log_level);

    HttpServer::new(move || {
        // Read configuration from environment
        let config = get_configuration();
        
        let game_manager = Data::new(GameManager::new(config.clone()).start());

        App::new()
            .app_data(Data::new(config))
            .app_data(game_manager.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())

            //Websocket
            .route("/v1/socket/", web::get().to(websocket_endpoint))

    })
    .bind(server_bind)?
    .run()
    .await
}
