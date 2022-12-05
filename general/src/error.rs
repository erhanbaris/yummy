use actix_web::{ResponseError, http::StatusCode, HttpResponse, HttpResponseBuilder, body::BoxBody};
use thiserror::Error;

use crate::web::GenericAnswer;

#[derive(Error, Debug)]
pub enum YummyError {
    #[error("{0}")]
    ActixError(#[from] actix_web::Error),

    #[error("{0}")]
    AnyHow(#[from] anyhow::Error)
}

impl ResponseError for YummyError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponseBuilder::new(self.status_code()).json(GenericAnswer {
            status: false,
            result: Some(self.to_string()),
        })
    }
}
