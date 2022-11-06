//mod websocket;
mod auth;
mod model;
mod config;

use std::{env, sync::{Arc, atomic::{AtomicUsize, AtomicU64, Ordering}}};
use actix_web::{middleware, App, HttpServer, web};
use uuid::Uuid;
use crate::config::HASHER;

//use crate::websocket::websocket_endpoint;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut guest_indexer = Arc::new(AtomicU64::new(1));

    use std::time::Instant;
    let now = Instant::now();

    for i in 0..10_000_000 {
        Uuid::new_v4();
        //let user_id = guest_indexer.fetch_add(1, Ordering::SeqCst);
        //HASHER.encode(&[user_id]);
    }

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    let server_bind = env::var("SERVER_BIND").unwrap_or_else(|_| "0.0.0.0:9090".to_string());
    let rust_log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info,backend,actix_web=info".to_string());

    std::env::set_var("RUST_LOG", &rust_log_level);
    env_logger::init();

    log::info!("Yummy Starting...");
    log::info!("Binding at \"{}\"", server_bind);
    log::info!("Rust log level is \"{}\"", rust_log_level);

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())

            //Websocket
            //.route("/v1/socket/", web::get().to(websocket_endpoint))

    })
    .bind(server_bind)?
    .run()
    .await
}
