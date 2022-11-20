use actix_web::{ResponseError, http::StatusCode, HttpResponse, HttpResponseBuilder, body::BoxBody};
use thiserror::Error;

use crate::web::GenericAnswer;

#[derive(Error, Debug)]
pub enum YummyError {
    #[error("Websocket connection arguments not valid. {0}")]
    WebsocketConnectArgument(String),

    #[error("{0}")]
    ActixError(#[from] actix_web::Error),

    #[error("{0}")]
    AnyHow(#[from] anyhow::Error),

    #[error("{0}")]
    DatabaseError(String),

    #[error("{0}")]
    IoError(#[from] std::io::Error),

    #[error("Unknown error")]
    Unknown,
}

impl ResponseError for YummyError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ActixError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Self::WebsocketConnectArgument(_) => StatusCode::BAD_REQUEST,
            Self::AnyHow(_) => StatusCode::BAD_REQUEST,
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponseBuilder::new(self.status_code()).json(GenericAnswer {
            status: false,
            result: Some(self.to_string()),
        })
    }
}

unsafe impl Send for YummyError {}
unsafe impl Sync for YummyError {}
