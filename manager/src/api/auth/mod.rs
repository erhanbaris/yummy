pub mod model;

use std::ops::Deref;
use general::{auth::{generate_auth, UserJwt, validate_auth}, model::YummyState};
use std::marker::PhantomData;
use std::sync::Arc;
use std::collections::HashMap;
use general::config::YummyConfig;

use actix::{Context, Handler, Actor, AsyncContext, SpawnHandle};
use database::{RowId, Pool, DatabaseTrait};
use anyhow::anyhow;
use general::model::{UserId, SessionId};

use crate::response::Response;

use self::model::*;

pub struct AuthManager<DB: DatabaseTrait + ?Sized> {
    config: Arc<YummyConfig>,
    database: Arc<Pool>,
    states: Arc<YummyState>,
    timeout_timers: HashMap<SessionId, SpawnHandle>,
    _auth: PhantomData<DB>
}

impl<DB: DatabaseTrait + ?Sized> AuthManager<DB> {
    pub fn new(config: Arc<YummyConfig>, states: Arc<YummyState>, database: Arc<Pool>) -> Self {
        Self {
            config,
            database,
            states,
            timeout_timers: HashMap::new(),
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
    fn handle(&mut self, auth: EmailAuthRequest, _ctx: &mut Context<Self>) -> Self::Result {

        let mut connection = self.database.get()?;
        let user_info: Option<(RowId, Option<String>, String)> = DB::user_login_via_email(&mut connection, &auth.email)?;

        let (user_id, name) = match (user_info, auth.if_not_exist_create) {
            (Some((user_id, name, password)), _) => {
                if auth.password != password {
                    return Err(anyhow!(AuthError::EmailOrPasswordNotValid));
                }

                DB::update_last_login(&mut connection, user_id)?;
                (user_id, name)
            },
            (None, true) => (DB::create_user_via_email(&mut connection, &auth.email, &auth.password)?, None),
            _ => return Err(anyhow!(AuthError::EmailOrPasswordNotValid))
        };
        
        if self.states.is_user_online(&UserId(user_id.0)) {
            return Err(anyhow!(AuthError::OnlyOneConnectionAllowedPerUser));
        }

        let session_id = self.states.new_session(UserId(user_id.0));
        self.generate_token(UserId(user_id.0), name, Some(auth.email.to_string()), Some(session_id))
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<DeviceIdAuthRequest> for AuthManager<DB> {
    type Result = anyhow::Result<Response>;

    #[tracing::instrument(name="Auth::ViaDeviceId", skip(self, _ctx))]
    fn handle(&mut self, auth: DeviceIdAuthRequest, _ctx: &mut Context<Self>) -> Self::Result {

        let mut connection = self.database.get()?;
        let user_info: Option<(RowId, Option<String>, Option<String>)> = DB::user_login_via_device_id(&mut connection, &auth.id)?;

        let (user_id, name, email) = match user_info {
            Some((user_id, name, email)) => (user_id, name, email),
            None => (DB::create_user_via_device_id(&mut connection, &auth.id)?, None, None)
        };
        
        if self.states.is_user_online(&UserId(user_id.0)) {
            return Err(anyhow!(AuthError::OnlyOneConnectionAllowedPerUser));
        }
        
        let session_id = self.states.new_session(UserId(user_id.0));
        self.generate_token(UserId(user_id.0), name, email, Some(session_id))
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<CustomIdAuthRequest> for AuthManager<DB> {
    type Result = anyhow::Result<Response>;

    #[tracing::instrument(name="Auth::ViaCustomId", skip(self, _ctx))]
    fn handle(&mut self, auth: CustomIdAuthRequest, _ctx: &mut Context<Self>) -> Self::Result {

        let mut connection = self.database.get()?;
        let user_info: Option<(RowId, Option<String>, Option<String>)> = DB::user_login_via_custom_id(&mut connection, &auth.id)?;

        let (user_id, name, email) = match user_info {
            Some((user_id, name, email)) => (user_id, name, email),
            None => (DB::create_user_via_custom_id(&mut connection, &auth.id)?, None, None)
        };
        
        if self.states.is_user_online(&UserId(user_id.0)) {
            return Err(anyhow!(AuthError::OnlyOneConnectionAllowedPerUser));
        }
        
        let session_id = self.states.new_session(UserId(user_id.0));
        self.generate_token(UserId(user_id.0), name, email, Some(session_id))
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<LogoutRequest> for AuthManager<DB> {
    type Result = anyhow::Result<Response>;

    #[tracing::instrument(name="Auth::Logout", skip(self, _ctx))]
    fn handle(&mut self, model: LogoutRequest, _ctx: &mut Context<Self>) -> Self::Result {
        match model.user.deref() {
            Some(user) => {
                self.states.as_ref().close_session(&user.session);
                Ok(Response::None)
            },
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<RefreshTokenRequest> for AuthManager<DB> {
    type Result = anyhow::Result<Response>;

    #[tracing::instrument(name="Auth::Refresh", skip(self, _ctx))]
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
    fn handle(&mut self, token: RestoreTokenRequest, ctx: &mut Context<Self>) -> Self::Result {
        match validate_auth(self.config.clone(), token.token) {
            Some(auth) => {
                let session_id = if self.states.is_session_online(&auth.user.session) {
                    if let Some(handle) = self.timeout_timers.remove(&auth.user.session) {
                        ctx.cancel_future(handle);
                    }
                    auth.user.session
                } else {
                    self.states.new_session(auth.user.id)
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
    fn handle(&mut self, model: StartUserTimeout, ctx: &mut Context<Self>) -> Self::Result {
        let session_id = model.session_id.clone();
        let timer = ctx.run_later(self.config.connection_restore_wait_timeout, move |manager, _ctx| {
            println!("User connection '{:?}' timed out", model.session_id);
            manager.states.close_session(model.session_id);
        });

        self.timeout_timers.insert(session_id, timer);

        Ok(Response::None)
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<StopUserTimeout> for AuthManager<DB> {
    type Result = anyhow::Result<Response>;

    #[tracing::instrument(name="Auth::StopTimer", skip(self, ctx))]
    fn handle(&mut self, model: StopUserTimeout, ctx: &mut Context<Self>) -> Self::Result {
        if let Some(handle) = self.timeout_timers.remove(&model.session_id) {
            ctx.cancel_future(handle);
        }

        Ok(Response::None)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use general::config::YummyConfig;
    use general::auth::validate_auth;
    use general::model::YummyState;
    use std::sync::Arc;

    use actix::Actor;
    use actix::Addr;
    use anyhow::Ok;
    use database::{create_database, create_connection};
    
    use super::AuthManager;
    use super::*;

    use crate::response::Response;

    fn create_actor(config: Arc<YummyConfig>) -> anyhow::Result<Addr<AuthManager<database::SqliteStore>>> {
        let connection = create_connection(":memory:")?;
        let states = Arc::new(YummyState::default());
        create_database(&mut connection.clone().get()?)?;
        Ok(AuthManager::<database::SqliteStore>::new(config.clone(), states, Arc::new(connection)).start())
    }

    /* email unit tests */
    #[actix::test]
    async fn create_user_via_email() -> anyhow::Result<()> {
        let address = create_actor(::general::config::get_configuration())?;
        address.send(EmailAuthRequest {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: true
        }).await??;
        Ok(())
    }

    #[actix::test]
    async fn login_user_via_email() -> anyhow::Result<()> {
        let mut config = ::general::config::get_configuration().deref().clone();
        config.connection_restore_wait_timeout = Duration::from_secs(1);
        config.heartbeat_interval = Duration::from_secs(1);
        config.client_timeout = Duration::from_secs(1);
        
        let address = create_actor(Arc::new(config))?;
        let response = address.send(EmailAuthRequest {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: true
        }).await??;

        if let Response::Auth(_, auth) = response.clone() {
            address.send(StartUserTimeout {
                session_id: auth.session.clone()
            }).await??;

            actix::clock::sleep(std::time::Duration::new(3, 0)).await;

            address.send(EmailAuthRequest {
                email: "erhanbaris@gmail.com".to_string(),
                password: "erhan".to_string(),
                if_not_exist_create: false
            }).await??;

            return Ok(());
        }

        return Err(anyhow::anyhow!("Unexpected response"));
    }

    #[actix::test]
    async fn failed_login_user_via_email_1() -> anyhow::Result<()> {
        let address = create_actor(::general::config::get_configuration())?;
        let result = address.send(EmailAuthRequest {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: false
        }).await?;

        assert_eq!(result.unwrap_err().to_string(), "Email and/or password not valid".to_string());
        Ok(())
    }

    #[actix::test]
    async fn failed_login_user_via_email_2() -> anyhow::Result<()> {
        let address = create_actor(::general::config::get_configuration())?;
        address.send(EmailAuthRequest {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: true
        }).await??;

        let result = address.send(EmailAuthRequest {
            email: "erhanbaris@gmail.com".to_string(),
            password: "wrong password".to_string(),
            if_not_exist_create: true
        }).await?;

        assert_eq!(result.unwrap_err().to_string(), "Email and/or password not valid".to_string());
        Ok(())
    }

    /* device id unit tests */
    #[actix::test]
    async fn create_user_via_device_id() -> anyhow::Result<()> {
        let address = create_actor(::general::config::get_configuration())?;
        address.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
        Ok(())
    }

    #[actix::test]
    async fn login_user_via_device_id() -> anyhow::Result<()> {
        let mut config = ::general::config::get_configuration().deref().clone();
        config.connection_restore_wait_timeout = Duration::from_secs(1);
        config.heartbeat_interval = Duration::from_secs(1);
        config.client_timeout = Duration::from_secs(1);
        
        let address = create_actor(Arc::new(config))?;
        let created_token = address.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;

        if let Response::Auth(_, auth) = created_token.clone() {
            address.send(StartUserTimeout {
                session_id: auth.session.clone()
            }).await??;

            actix::clock::sleep(std::time::Duration::new(3, 0)).await;

            let logged_in_token = address.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
            assert_ne!(created_token, logged_in_token);
    
            return Ok(());
        }

        return Err(anyhow::anyhow!("Unexpected response"));
    }

    #[actix::test]
    async fn login_users_via_device_id() -> anyhow::Result<()> {
        let address = create_actor(::general::config::get_configuration())?;
        let login_1 = address.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
        let login_2 = address.send(DeviceIdAuthRequest::new("abcdef".to_string())).await??;
        assert_ne!(login_1, login_2);

        Ok(())
    }

    /* custom id unit tests */
    #[actix::test]
    async fn create_user_via_custom_id() -> anyhow::Result<()> {
        let address = create_actor(::general::config::get_configuration())?;
        address.send(CustomIdAuthRequest::new("1234567890".to_string())).await??;
        Ok(())
    }

    #[actix::test]
    async fn login_user_via_custom_id() -> anyhow::Result<()> {
        let mut config = ::general::config::get_configuration().deref().clone();
        config.connection_restore_wait_timeout = Duration::from_secs(1);
        config.heartbeat_interval = Duration::from_secs(1);
        config.client_timeout = Duration::from_secs(1);
        
        let address = create_actor(Arc::new(config))?;
        let created_token = address.send(CustomIdAuthRequest::new("1234567890".to_string())).await??;

        if let Response::Auth(_, auth) = created_token.clone() {
            address.send(StartUserTimeout {
                session_id: auth.session.clone()
            }).await??;

            actix::clock::sleep(std::time::Duration::new(3, 0)).await;

            let logged_in_token = address.send(CustomIdAuthRequest::new("1234567890".to_string())).await??;
            assert_ne!(created_token, logged_in_token);
    
            return Ok(());
        }

        return Err(anyhow::anyhow!("Unexpected response"));
    }

    #[actix::test]
    async fn login_users_via_custom_id() -> anyhow::Result<()> {
        let address = create_actor(::general::config::get_configuration())?;
        let login_1 = address.send(CustomIdAuthRequest::new("1234567890".to_string())).await??;
        let login_2 = address.send(CustomIdAuthRequest::new("abcdef".to_string())).await??;
        assert_ne!(login_1, login_2);

        Ok(())
    }

    /* restore token unit tests */
    #[actix::test]
    async fn token_restore_test_1() -> anyhow::Result<()> {
        let config = ::general::config::get_configuration();
        let address = create_actor(config.clone())?;
        let old_token: Response = address.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
        let old_token = match old_token {
            Response::Auth(token, _) => token,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
        };

        // Wait 1 second
        actix::clock::sleep(std::time::Duration::new(1, 0)).await;
        let new_token: Response = address.send(RestoreTokenRequest { token: old_token.to_string() }).await??;
        let new_token = match new_token {
            Response::Auth(token, _) => token,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
        };
       
        assert_ne!(old_token.clone(), new_token.clone());

        let old_claims =  validate_auth(config.clone(), old_token).unwrap();
        let new_claims =  validate_auth(config.clone(), new_token).unwrap();

        assert_eq!(old_claims.user.id.clone(), new_claims.user.id.clone());
        assert_eq!(old_claims.user.name.clone(), new_claims.user.name.clone());
        assert_eq!(old_claims.user.email.clone(), new_claims.user.email.clone());
        assert_eq!(old_claims.user.session.clone(), new_claims.user.session.clone());

        assert!(old_claims.exp < new_claims.exp);

        Ok(())
    }

    #[actix::test]
    async fn fail_token_restore_test_1() -> anyhow::Result<()> {
        let mut config = ::general::config::get_configuration().deref().clone();
        config.token_lifetime = Duration::from_secs(1);

        let address = create_actor(Arc::new(config))?;
        let old_token: Response = address.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
        let old_token = match old_token {
            Response::Auth(token, _) => token,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
        };

        // Wait 3 seconds
        actix::clock::sleep(std::time::Duration::new(3, 0)).await;
        let response = address.send(RestoreTokenRequest { token: old_token.to_string() }).await?;
        
        if response.is_ok() {
            assert!(false, "Expected exception, received: {:?}", response);
        }
        
        Ok(())
    }

    /* refreh token unit tests */
    #[actix::test]
    async fn token_refresh_test_1() -> anyhow::Result<()> {
        let config = ::general::config::get_configuration();
        let address = create_actor(config.clone())?;
        let old_token: Response = address.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
        let old_token = match old_token {
            Response::Auth(token, _) => token,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
        };

        // Wait 1 second
        actix::clock::sleep(std::time::Duration::new(1, 0)).await;
        let new_token: Response = address.send(RefreshTokenRequest { token: old_token.to_string() }).await??;
        let new_token = match new_token {
            Response::Auth(token, _) => token,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
        };
       
        assert_ne!(old_token.clone(), new_token.clone());

        let old_claims =  validate_auth(config.clone(), old_token).unwrap();
        let new_claims =  validate_auth(config.clone(), new_token).unwrap();

        assert_eq!(old_claims.user.id.clone(), new_claims.user.id.clone());
        assert_eq!(old_claims.user.name.clone(), new_claims.user.name.clone());
        assert_eq!(old_claims.user.email.clone(), new_claims.user.email.clone());
        assert_eq!(old_claims.user.session.clone(), new_claims.user.session.clone());

        assert!(old_claims.exp < new_claims.exp);

        Ok(())
    }

    #[actix::test]
    async fn token_refresh_test_2() -> anyhow::Result<()> {
        let config = ::general::config::get_configuration();
        let address = create_actor(config.clone())?;
        let old_token: Response = address.send(EmailAuthRequest {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: true
        }).await??;

        let old_token = match old_token {
            Response::Auth(token, _) => token,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
        };

        // Wait 1 second
        actix::clock::sleep(std::time::Duration::new(1, 0)).await;
        let new_token: Response = address.send(RefreshTokenRequest{ token: old_token.clone() }).await??;
        let new_token = match new_token {
            Response::Auth(token, _) => token,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
        };
       
        assert_ne!(old_token.clone(), new_token.clone());

        let old_claims =  validate_auth(config.clone(), old_token).unwrap();
        let new_claims =  validate_auth(config.clone(), new_token).unwrap();

        assert_eq!(old_claims.user.id.clone(), new_claims.user.id.clone());
        assert_eq!(old_claims.user.name.clone(), new_claims.user.name.clone());
        assert_eq!(old_claims.user.email.clone(), new_claims.user.email.clone());
        assert_eq!(old_claims.user.session.clone(), new_claims.user.session.clone());

        assert!(old_claims.exp < new_claims.exp);

        Ok(())
    }
}