use actix_web::{ResponseError, http::StatusCode};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum YummyError {
    #[error("Websocket connection arguments not valid. {0}")]
    WebsocketConnectArgument(String),

    #[error("Internal error. {0}")]
    ActixError(#[from] actix_web::Error),

    #[error("unknown data store error")]
    Unknown,
}

impl ResponseError for YummyError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ActixError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Self::WebsocketConnectArgument(_) => StatusCode::BAD_REQUEST
        }
    }
}
