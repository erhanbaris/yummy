pub mod model;

#[cfg(test)]
mod test;

use std::ops::Deref;
use actix_broker::BrokerIssue;
use general::{auth::{generate_auth, UserJwt, validate_auth}, model::YummyState};
use std::marker::PhantomData;
use std::sync::Arc;
use std::collections::HashMap;
use general::config::YummyConfig;

use actix::{Context, Handler, Actor, AsyncContext, SpawnHandle};
use database::{Pool, DatabaseTrait};
use anyhow::anyhow;
use general::model::{UserId, SessionId};

use crate::response::Response;

use self::model::*;

pub struct AuthManager<DB: DatabaseTrait + ?Sized> {
    config: Arc<YummyConfig>,
    database: Arc<Pool>,
    states: Arc<YummyState>,
    session_timeout_timers: HashMap<SessionId, SpawnHandle>,
    _auth: PhantomData<DB>
}

impl<DB: DatabaseTrait + ?Sized> AuthManager<DB> {
    pub fn new(config: Arc<YummyConfig>, states: Arc<YummyState>, database: Arc<Pool>) -> Self {
        Self {
            config,
            database,
            states,
            session_timeout_timers: HashMap::new(),
            _auth: PhantomData
        }
    }

    pub fn generate_token(&self, id: UserId, name: Option<String>, email: Option<String>, session: Option<SessionId>) -> anyhow::Result<Response> {
        let user_jwt = UserJwt {
            id,
            session: session.unwrap_or_else(SessionId::new) ,
            name,
            email
        };

        let token = match generate_auth(self.config.clone(), &user_jwt) {
            Some(token) => token,
            _ => return Err(anyhow::anyhow!(AuthError::TokenCouldNotGenerated))
        };

        Ok(Response::Auth(token, user_jwt))
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Actor for AuthManager<DB> {
    type Context = Context<Self>;
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<EmailAuthRequest> for AuthManager<DB> {
    type Result = anyhow::Result<Response>;

    #[tracing::instrument(name="Auth::ViaEmail", skip(self, _ctx))]
    #[macros::api(name="ViaEmail")]
    fn handle(&mut self, model: EmailAuthRequest, _ctx: &mut Context<Self>) -> Self::Result {

        let mut connection = self.database.get()?;
        let user_info = DB::user_login_via_email(&mut connection, &model.email)?;

        let (user_id, name) = match (user_info, model.if_not_exist_create) {
            (Some(user_info), _) => {
                if model.password != user_info.password.unwrap_or_default() {
                    return Err(anyhow!(AuthError::EmailOrPasswordNotValid));
                }

                DB::update_last_login(&mut connection, user_info.user_id)?;
                (user_info.user_id, user_info.name)
            },
            (None, true) => (DB::create_user_via_email(&mut connection, &model.email, &model.password)?, None),
            _ => return Err(anyhow!(AuthError::EmailOrPasswordNotValid))
        };
        
        if self.states.is_user_online(UserId::from(user_id.get())) {
            return Err(anyhow!(AuthError::OnlyOneConnectionAllowedPerUser));
        }

        let session_id = self.states.new_session(UserId::from(user_id.get()), model.socket.clone());
        self.generate_token(UserId::from(user_id.get()), name, Some(model.email.to_string()), Some(session_id))
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<DeviceIdAuthRequest> for AuthManager<DB> {
    type Result = anyhow::Result<Response>;

    #[tracing::instrument(name="Auth::ViaDeviceId", skip(self, _ctx))]
    #[macros::api(name="ViaEmail")]
    fn handle(&mut self, auth: DeviceIdAuthRequest, _ctx: &mut Context<Self>) -> Self::Result {

        let mut connection = self.database.get()?;
        let user_info = DB::user_login_via_device_id(&mut connection, &auth.id)?;

        let (user_id, name, email) = match user_info {
            Some(user_info) => (user_info.user_id, user_info.name, user_info.email),
            None => (DB::create_user_via_device_id(&mut connection, &auth.id)?, None, None)
        };
        
        if self.states.is_user_online(UserId::from(user_id.get())) {
            return Err(anyhow!(AuthError::OnlyOneConnectionAllowedPerUser));
        }
        
        let session_id = self.states.new_session(UserId::from(user_id.get()), auth.socket.clone());
        self.generate_token(UserId::from(user_id.get()), name, email, Some(session_id))
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<CustomIdAuthRequest> for AuthManager<DB> {
    type Result = anyhow::Result<Response>;

    #[tracing::instrument(name="Auth::ViaCustomId", skip(self, _ctx))]
    #[macros::api(name="ViaEmail")]
    fn handle(&mut self, auth: CustomIdAuthRequest, _ctx: &mut Context<Self>) -> Self::Result {

        let mut connection = self.database.get()?;
        let user_info = DB::user_login_via_custom_id(&mut connection, &auth.id)?;

        let (user_id, name, email) = match user_info {
            Some(user_info) => (user_info.user_id, user_info.name, user_info.email),
            None => (DB::create_user_via_custom_id(&mut connection, &auth.id)?, None, None)
        };
        
        if self.states.is_user_online(UserId::from(user_id.get())) {
            return Err(anyhow!(AuthError::OnlyOneConnectionAllowedPerUser));
        }
        
        let session_id = self.states.new_session(UserId::from(user_id.get()), auth.socket.clone());
        self.generate_token(UserId::from(user_id.get()), name, email, Some(session_id))
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<LogoutRequest> for AuthManager<DB> {
    type Result = anyhow::Result<Response>;

    #[tracing::instrument(name="Auth::Logout", skip(self, _ctx))]
    #[macros::api(name="ViaEmail")]
    fn handle(&mut self, model: LogoutRequest, _ctx: &mut Context<Self>) -> Self::Result {
        match model.user.deref() {
            Some(user) => {
                self.states.as_ref().close_session(&user.session);
                Ok(Response::None)
            },
            None => Err(anyhow::anyhow!(AuthError::TokenNotValid))
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<RefreshTokenRequest> for AuthManager<DB> {
    type Result = anyhow::Result<Response>;

    #[tracing::instrument(name="Auth::Refresh", skip(self, _ctx))]
    #[macros::api(name="ViaEmail")]
    fn handle(&mut self, token: RefreshTokenRequest, _ctx: &mut Context<Self>) -> Self::Result {
        match validate_auth(self.config.clone(), token.token) {
            Some(claims) => self.generate_token(claims.user.id, claims.user.name, claims.user.email, Some(claims.user.session)),
            None => Err(anyhow!(AuthError::TokenNotValid))
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<RestoreTokenRequest> for AuthManager<DB> {
    type Result = anyhow::Result<Response>;

    #[tracing::instrument(name="Auth::Restore", skip(self, ctx))]
    #[macros::api(name="ViaEmail")]
    fn handle(&mut self, token: RestoreTokenRequest, ctx: &mut Context<Self>) -> Self::Result {
        match validate_auth(self.config.clone(), token.token) {
            Some(auth) => {
                let session_id = if self.states.is_session_online(&auth.user.session) {
                    if let Some(handle) = self.session_timeout_timers.remove(&auth.user.session) {
                        ctx.cancel_future(handle);
                    }
                    auth.user.session
                } else {
                    self.states.new_session(auth.user.id, token.socket.clone())
                };

                self.generate_token(auth.user.id, None, None, Some(session_id))
            },
            None => Err(anyhow!(AuthError::TokenNotValid))
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<StartUserTimeout> for AuthManager<DB> {
    type Result = anyhow::Result<Response>;

    #[tracing::instrument(name="Auth::StartTimer", skip(self, ctx))]
    #[macros::api(name="ViaEmail")]
    fn handle(&mut self, model: StartUserTimeout, ctx: &mut Context<Self>) -> Self::Result {
        let session_id = model.session_id.clone();
        let timer = ctx.run_later(self.config.connection_restore_wait_timeout, move |manager, _ctx| {
            if let Some(user) = manager.states.close_session(&model.session_id) {
                manager.issue_system_async(UserDisconnectRequest {
                    user_id: user.user_id
                });
            }
        });

        self.session_timeout_timers.insert(session_id, timer);
        Ok(Response::None)
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<StopUserTimeout> for AuthManager<DB> {
    type Result = anyhow::Result<Response>;

    #[tracing::instrument(name="Auth::StopTimer", skip(self, ctx))]
    #[macros::api(name="ViaEmail")]
    fn handle(&mut self, model: StopUserTimeout, ctx: &mut Context<Self>) -> Self::Result {
        if let Some(handle) = self.session_timeout_timers.remove(&model.session_id) {
            ctx.cancel_future(handle);
        }

        Ok(Response::None)
    }
}
