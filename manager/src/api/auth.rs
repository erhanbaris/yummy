use actix::{Context, Handler};
use actix::prelude::Message;
use database::RowId;
use database::auth::AuthStore;
use thiserror::Error;
use uuid::Uuid;
use anyhow::anyhow;

use core::model::SessionId;

use secrecy::{ExposeSecret, SecretString};

use crate::GameManager;

#[derive(Message, Debug)]
#[rtype(result = "anyhow::Result<SessionId>")]
pub struct EmailAuth {
    pub email: String,
    pub password: SecretString,
    pub if_not_exist_create: bool
}

unsafe impl Send for EmailAuth {}
unsafe impl Sync for EmailAuth {}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Email and/or password not valid")]
    EmailOrPasswordNotValid
}

impl Handler<EmailAuth> for GameManager {
    type Result = anyhow::Result<SessionId>;

    #[tracing::instrument(name="Manager::EmailAuth", skip(self, _ctx))]
    fn handle(&mut self, auth: EmailAuth, _ctx: &mut Context<Self>) -> Self::Result {

        let mut auth_store = AuthStore::new(self.database.get()?);
        let user_info: Option<(RowId, SecretString)> = auth_store.user_login_via_email(&auth.email)?;

        let user_id = match (user_info, auth.if_not_exist_create) {
            (Some((user_id, password)), _) => {
                if auth.password.expose_secret() != password.expose_secret() {
                    return Err(anyhow!(AuthError::EmailOrPasswordNotValid));
                }

                user_id
            },
            (None, true) => auth_store.create_user_via_email(&auth.email, &auth.password)?,
            _ => return Err(anyhow!(AuthError::EmailOrPasswordNotValid))
        };

        Ok(SessionId(user_id.0))
    }
}
