use actix::{Context, Handler};
use actix::prelude::Message;

use core::error::YummyError;
use core::model::SessionId;

use secrecy::SecretString;

use crate::GameManager;

#[derive(Message)]
#[rtype(result = "Result<SessionId, YummyError>")]
pub struct EmailAuth {
    pub email: String,
    pub password: SecretString,
    pub if_not_exist_create: bool
}

unsafe impl Send for EmailAuth {}
unsafe impl Sync for EmailAuth {}

impl Handler<EmailAuth> for GameManager {
    type Result = Result<SessionId, YummyError>;

    fn handle(&mut self, _: EmailAuth, _: &mut Context<Self>) -> Self::Result {
        Ok(SessionId::default())
    }
}
