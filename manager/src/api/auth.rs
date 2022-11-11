use core::jwt::{generate_auth, UserJwt, validate_auth};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use core::config::YummyConfig;

use actix::{Context, Handler, Actor};
use actix::prelude::Message;
use database::{RowId, Pool};
use database::auth::{AuthStore, AuthStoreTrait};
use thiserror::Error;
use anyhow::anyhow;

use core::model::{SessionToken, UserId};

use secrecy::{ExposeSecret, SecretString};

#[derive(Message, Debug)]
#[rtype(result = "anyhow::Result<SessionToken>")]
pub struct EmailAuth {
    pub email: String,
    pub password: SecretString,
    pub if_not_exist_create: bool,
}

unsafe impl Send for EmailAuth {}
unsafe impl Sync for EmailAuth {}

#[derive(Message, Debug)]
#[rtype(result = "anyhow::Result<SessionToken>")]
pub struct RefreshToken {
    pub token: String
}

unsafe impl Send for RefreshToken {}
unsafe impl Sync for RefreshToken {}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Email and/or password not valid")]
    EmailOrPasswordNotValid,

    #[error("Session token could not generated")]
    TokenCouldNotGenerated,
    
    #[error("Token is not valid")]
    TokenNotValid
}

pub struct AuthManager<A: AuthStoreTrait> {
    config: Arc<YummyConfig>,
    database: Arc<Pool>,
    _auth: PhantomData<A>
}

impl<A: AuthStoreTrait> AuthManager<A> {
    pub fn new(config: Arc<YummyConfig>, database: Arc<Pool>) -> Self {
        Self {
            config,
            database,
            _auth: PhantomData
        }
    }

    pub fn generate_token(&self, id: UserId, name: Option<String>, email: Option<String>) -> anyhow::Result<SessionToken> {
        match generate_auth(self.config.clone(), UserJwt {
            id,
            name,
            email
        }) {
            Some(token) => Ok(SessionToken(token)),
            _ => Err(anyhow::anyhow!(AuthError::TokenCouldNotGenerated))
        }
    }
}

impl<A: AuthStoreTrait + std::marker::Unpin + 'static> Actor for AuthManager<A> {
    type Context = Context<Self>;
}

impl<A: AuthStoreTrait + std::marker::Unpin + 'static> Handler<EmailAuth> for AuthManager<A> {
    type Result = anyhow::Result<SessionToken>;

    #[tracing::instrument(name="Auth::ViaEmail", skip(self, _ctx))]
    fn handle(&mut self, auth: EmailAuth, _ctx: &mut Context<Self>) -> Self::Result {

        let mut auth_store = AuthStore::new(self.database.get()?);
        let user_info: Option<(RowId, Option<String>, SecretString)> = auth_store.user_login_via_email(&auth.email)?;

        let (user_id, name) = match (user_info, auth.if_not_exist_create) {
            (Some((user_id, name, password)), _) => {
                if auth.password.expose_secret() != password.expose_secret() {
                    return Err(anyhow!(AuthError::EmailOrPasswordNotValid));
                }

                (user_id, name)
            },
            (None, true) => (auth_store.create_user_via_email(&auth.email, &auth.password)?, None),
            _ => return Err(anyhow!(AuthError::EmailOrPasswordNotValid))
        };
        
        let response = self.generate_token(UserId(user_id.0), name, Some(auth.email.to_string()))?;
        Ok(response)
    }
}

impl<A: AuthStoreTrait + std::marker::Unpin + 'static> Handler<RefreshToken> for AuthManager<A> {
    type Result = anyhow::Result<SessionToken>;

    #[tracing::instrument(name="Manager::Refresh", skip(self, _ctx))]
    fn handle(&mut self, token: RefreshToken, _ctx: &mut Context<Self>) -> Self::Result {
        match validate_auth(self.config.clone(), token.token) {
            Some(claims) => self.generate_token(claims.user.id, claims.user.name, claims.user.email),
            None => Err(anyhow!(AuthError::TokenNotValid))
        }
    }
}
