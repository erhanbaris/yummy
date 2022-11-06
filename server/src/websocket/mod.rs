mod socket;

use actix_web::{HttpRequest, web};
use actix_web_actors::ws;

use crate::auth::{validate_auth, UserJwt};
use crate::websocket::socket::{GameWebsocket, ConnectionInfo};

pub async fn websocket_endpoint(req: HttpRequest, stream: web::Payload, connnection_info: Option<web::Query<ConnectionInfo>>) -> Result<actix_web::HttpResponse, actix_web::Error> {
    println!("Connecting: {:?}", connnection_info);

    let (connection_id, connection_key) = match connnection_info {
        Some(info) => (info.id.unwrap_or_default(), info.key.to_owned()),
        None => (0, String::new()),
    };

    let (player, valid) = match validate_auth(&connection_key[..]) {
        Some(auth) => (auth.user, true),
        None => (UserJwt::default(), false),
    };

    ws::start(GameWebsocket::new(connection_id, player, manager.get_ref().clone(), valid), &req, stream)
}