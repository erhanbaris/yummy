use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Answer {
    pub status: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GenericAnswer<T> {
    pub status: bool,
    pub result: Option<T>,
}