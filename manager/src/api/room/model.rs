use std::{fmt::Debug, sync::Arc};

use actix::prelude::Message;
use thiserror::Error;
use validator::Validate;

use general::{auth::UserAuth, model::CreateRoomAccessType};

use crate::response::Response;

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<Response>")]
pub struct CreateRoomRequest {
    pub user: Arc<Option<UserAuth>>,
    pub disconnect_from_other_room: bool,
    pub name: Option<String>,
    pub access_type: CreateRoomAccessType,
    pub max_user: usize,
    pub tags: Vec<String>
}

#[derive(Error, Debug)]
pub enum RoomError {
    #[error("User joined to other room")]
    UserJoinedOtherRoom
}