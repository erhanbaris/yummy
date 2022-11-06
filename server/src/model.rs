use actix::MessageResponse;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Default, MessageResponse, Deserialize, Serialize, Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct UserId(pub u64);
