pub mod model;

#[cfg(test)]
mod test;

use std::{ops::Deref, fmt::Debug};
use actix_broker::BrokerIssue;
use general::{auth::{generate_auth, UserJwt, validate_auth}, state::YummyState, web::GenericAnswer};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;
use std::sync::Arc;
use std::collections::HashMap;
use general::config::YummyConfig;

use actix::{Context, Handler, Actor, AsyncContext, SpawnHandle};
use database::{Pool, DatabaseTrait};
use anyhow::{anyhow, Ok};
use general::model::{UserId, SessionId};

use self::model::*;
use crate::api::conn::model::UserConnected;

pub fn generate_response<T: Debug + Serialize + DeserializeOwned>(model: T) -> String {
    serde_json::to_string(&model).unwrap_or_default()
}

pub struct AuthManager<DB: DatabaseTrait + ?Sized> {
    config: Arc<YummyConfig>,
    database: Arc<Pool>,
    states: YummyState,
    session_timeout_timers: HashMap<SessionId, SpawnHandle>,
    _auth: PhantomData<DB>
}

impl<DB: DatabaseTrait + ?Sized> AuthManager<DB> {
    pub fn new(config: Arc<YummyConfig>, states: YummyState, database: Arc<Pool>) -> Self {
        Self {
            config,
            database,
            states,
            session_timeout_timers: HashMap::new(),
            _auth: PhantomData,
        }
    }

    pub fn generate_token(&self, id: UserId, name: Option<String>, email: Option<String>, session: Option<SessionId>) -> anyhow::Result<(String, UserJwt)> {
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

        Ok((token, user_jwt))
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Actor for AuthManager<DB> {
    type Context = Context<Self>;
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<EmailAuthRequest> for AuthManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="EmailAuth", skip(self, _ctx))]
    #[macros::api(name="EmailAuth", socket=true)]
    fn handle(&mut self, model: EmailAuthRequest, _ctx: &mut Context<Self>) -> Self::Result {
        let mut connection = self.database.get()?;
        let user_info = DB::user_login_via_email(&mut connection, &model.email)?;
        log::info!("{:?}", model);

        let (user_id, name) = match (user_info, model.if_not_exist_create) {
            (Some(user_info), _) => {
                if model.password.get() != &user_info.password.unwrap_or_default() {
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

        let session_id = self.states.new_session(UserId::from(user_id.get()), name.clone());
        let (token, auth) = self.generate_token(UserId::from(user_id.get()), name, Some(model.email.to_string()), Some(session_id))?;

        self.issue_system_async(UserConnected {
            user_id: UserId::from(user_id.get()),
            socket: model.socket.clone()
        });
        model.socket.authenticated(auth);
        model.socket.send(GenericAnswer::success(token).into());
        Ok(())
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<DeviceIdAuthRequest> for AuthManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="DeviceIdAuth", skip(self, _ctx))]
    #[macros::api(name="DeviceIdAuth", socket=true)]
    fn handle(&mut self, model: DeviceIdAuthRequest, _ctx: &mut Context<Self>) -> Self::Result {

        let mut connection = self.database.get()?;
        let user_info = DB::user_login_via_device_id(&mut connection, &model.id)?;

        let (user_id, name, email) = match user_info {
            Some(user_info) => (user_info.user_id, user_info.name, user_info.email),
            None => (DB::create_user_via_device_id(&mut connection, &model.id)?, None, None)
        };
        
        if self.states.is_user_online(UserId::from(user_id.get())) {
            return Err(anyhow!(AuthError::OnlyOneConnectionAllowedPerUser));
        }
        
        let session_id = self.states.new_session(UserId::from(user_id.get()), name.clone());
        let (token, auth) = self.generate_token(UserId::from(user_id.get()), name, email, Some(session_id))?;

        self.issue_system_async(UserConnected {
            user_id: UserId::from(user_id.get()),
            socket: model.socket.clone()
        });
        model.socket.authenticated(auth);
        model.socket.send(GenericAnswer::success(token).into());
        Ok(())
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<CustomIdAuthRequest> for AuthManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="CustomIdAuth", skip(self, _ctx))]
    #[macros::api(name="CustomIdAuth", socket=true)]
    fn handle(&mut self, model: CustomIdAuthRequest, _ctx: &mut Context<Self>) -> Self::Result {

        let mut connection = self.database.get()?;
        let user_info = DB::user_login_via_custom_id(&mut connection, &model.id)?;

        let (user_id, name, email) = match user_info {
            Some(user_info) => (user_info.user_id, user_info.name, user_info.email),
            None => (DB::create_user_via_custom_id(&mut connection, &model.id)?, None, None)
        };
        
        if self.states.is_user_online(UserId::from(user_id.get())) {
            return Err(anyhow!(AuthError::OnlyOneConnectionAllowedPerUser));
        }
        
        let session_id = self.states.new_session(UserId::from(user_id.get()), name.clone());
        let (token, auth) = self.generate_token(UserId::from(user_id.get()), name, email, Some(session_id))?;


        self.issue_system_async(UserConnected {
            user_id: UserId::from(user_id.get()),
            socket: model.socket.clone()
        });
        model.socket.authenticated(auth);
        model.socket.send(GenericAnswer::success(token).into());
        Ok(())
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<LogoutRequest> for AuthManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="Logout", skip(self, _ctx))]
    #[macros::api(name="Logout")]
    fn handle(&mut self, model: LogoutRequest, _ctx: &mut Context<Self>) -> Self::Result {
        match model.user.deref() {
            Some(user) => {
                self.states.close_session(&user.session);
                Ok(())
            },
            None => Err(anyhow::anyhow!(AuthError::TokenNotValid))
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<RefreshTokenRequest> for AuthManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="RefreshToken", skip(self, _ctx))]
    #[macros::api(name="RefreshToken", socket=true)]
    fn handle(&mut self, model: RefreshTokenRequest, _ctx: &mut Context<Self>) -> Self::Result {
        match validate_auth(self.config.clone(), model.token) {
            Some(claims) => {
                let (token, _) = self.generate_token(claims.user.id, claims.user.name, claims.user.email, Some(claims.user.session))?;
                model.socket.send(GenericAnswer::success(token).into());
                Ok(())
            },
            None => Err(anyhow!(AuthError::TokenNotValid))
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<RestoreTokenRequest> for AuthManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="RestoreToken", skip(self, ctx))]
    #[macros::api(name="RestoreToken", socket=true)]
    fn handle(&mut self, model: RestoreTokenRequest, ctx: &mut Context<Self>) -> Self::Result {
        match validate_auth(self.config.clone(), model.token) {
            Some(auth) => {
                let session_id = if self.states.is_session_online(&auth.user.session) {
                    if let Some(handle) = self.session_timeout_timers.remove(&auth.user.session) {
                        ctx.cancel_future(handle);
                    }
                    auth.user.session
                } else {
                    self.states.new_session(auth.user.id, auth.user.name.clone())
                };

                let (token, auth) = self.generate_token(auth.user.id, None, None, Some(session_id))?;

                self.issue_system_async(UserConnected {
                    user_id: auth.id.clone(),
                    socket: model.socket.clone()
                });
                model.socket.authenticated(auth);
                model.socket.send(GenericAnswer::success(token).into());
                Ok(())
            },
            None => Err(anyhow!(AuthError::TokenNotValid))
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<StartUserTimeout> for AuthManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="StartUserTimeout", skip(self, ctx))]
    #[macros::api(name="StartUserTimeout")]
    fn handle(&mut self, model: StartUserTimeout, ctx: &mut Context<Self>) -> Self::Result {
        let session_id = model.session_id.clone();
        let timer = ctx.run_later(self.config.connection_restore_wait_timeout, move |manager, _ctx| {
            if manager.states.close_session(&model.session_id) {
                manager.issue_system_async(UserDisconnectRequest {
                    user_id: model.user_id
                });
            }
        });

        self.session_timeout_timers.insert(session_id, timer);
        Ok(())
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<StopUserTimeout> for AuthManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="StopUserTimeout", skip(self, ctx))]
    #[macros::api(name="StopUserTimeout")]
    fn handle(&mut self, model: StopUserTimeout, ctx: &mut Context<Self>) -> Self::Result {
        if let Some(handle) = self.session_timeout_timers.remove(&model.session_id) {
            ctx.cancel_future(handle);
        }

        Ok(())
    }
}
