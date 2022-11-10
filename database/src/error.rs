#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Database error. {0}")]
    DieselError(#[from] diesel::result::Error),

    #[error("Database error. {0}")]
    R2d2Error(#[from] r2d2::Error),

    #[error("Uuid could not parsed. {0}")]
    UuidParseError(#[from] uuid::Error),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
