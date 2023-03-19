use actix_web::{ResponseError, http::StatusCode};
use thiserror::Error;

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
}
