use actix_web::{ResponseError, http::StatusCode, Responder, HttpResponse, HttpResponseBuilder, body::BoxBody};
use thiserror::Error;

use crate::web::GenericAnswer;

#[derive(Error, Debug)]
pub enum YummyError {
    #[error("Websocket connection arguments not valid. {0}")]
    WebsocketConnectArgument(String),

    #[error("Internal error. {0}")]
    ActixError(#[from] actix_web::Error),

    #[error("Internal error")]
    AnyHow(#[from] anyhow::Error),

    #[error("Unknown error")]
    Unknown,
}

impl From<YummyError> for HttpResponse {
    fn from(error: YummyError) -> Self {
        HttpResponseBuilder::new(error.status_code()).json(GenericAnswer {
            status: true,
            result: Some(error.to_string()),
        })
    }
}

impl ResponseError for YummyError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ActixError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Self::WebsocketConnectArgument(_) => StatusCode::BAD_REQUEST,
            Self::AnyHow(_) => StatusCode::BAD_REQUEST
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponseBuilder::new(self.status_code()).json(GenericAnswer {
            status: true,
            result: Some(self.to_string()),
        })
    }
}

unsafe impl Send for YummyError {}
unsafe impl Sync for YummyError {}
