use std::sync::Arc;

use actix::Addr;
use actix_web::web::Data;
use actix_web::{HttpRequest, web};
use actix_web_actors::ws;

use crate::config::YummyConfig;
use crate::error::YummyError;
use crate::game::GameManager;
use crate::websocket::request::ConnectionInfo;
use crate::websocket::GameWebsocket;

pub async fn websocket_endpoint(req: HttpRequest, stream: web::Payload, config: Data<Arc<YummyConfig>>, manager: web::Data<Addr<GameManager>>, connnection_info: Result<web::Query<ConnectionInfo>, actix_web::Error>) -> Result<actix_web::HttpResponse, YummyError> {
    log::debug!("Websocket connection: {:?}", connnection_info);
    let config = config.get_ref();

    let connnection_info = match connnection_info {
        Ok(connnection_info) => connnection_info.into_inner(),
        Err(error) => return Err(YummyError::WebsocketConnectArgument(error.to_string()))
    };

    ws::start(GameWebsocket::new(config.clone(), connnection_info, manager.get_ref().clone()), &req, stream)
        .map_err(|error| YummyError::from(error))
}
