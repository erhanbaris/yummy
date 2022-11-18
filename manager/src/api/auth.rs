use general::auth::{generate_auth, UserJwt, validate_auth};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use general::config::YummyConfig;

use actix::{Context, Handler, Actor};
use actix::prelude::Message;
use database::{RowId, Pool, DatabaseTrait};
use thiserror::Error;
use anyhow::anyhow;
use validator::Validate;

use general::model::{SessionToken, UserId, SessionId};

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<SessionToken>")]
pub struct EmailAuthRequest {
    #[validate(email(message="Email address is not valid"))]
    pub email: String,

    #[validate(length(min = 3, max = 32, message = "Length should be between 3 to 32 chars"))]
    pub password: String,

    pub if_not_exist_create: bool,
}

unsafe impl Send for EmailAuthRequest {}
unsafe impl Sync for EmailAuthRequest {}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<SessionToken>")]
pub struct RefreshTokenRequest {

    #[validate(length(min = 275, max = 1024, message = "Length should be between 275 to 1024 chars"))]
    pub token: String
}

impl From<SessionToken> for RefreshTokenRequest {
    fn from(token: SessionToken) -> Self {
        RefreshTokenRequest { token: token.0 }
    }
}

unsafe impl Send for RefreshTokenRequest {}
unsafe impl Sync for RefreshTokenRequest {}

#[derive(Message, Debug, Validate)]
#[rtype(result = "anyhow::Result<SessionToken>")]
pub struct DeviceIdAuthRequest {
    #[validate(length(min = 8, max = 128, message = "Length should be between 8 to 128 chars"))]
    pub id: String
}

impl DeviceIdAuthRequest {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

unsafe impl Send for DeviceIdAuthRequest {}
unsafe impl Sync for DeviceIdAuthRequest {}

#[derive(Message, Debug, Validate)]
#[rtype(result = "anyhow::Result<SessionToken>")]
pub struct CustomIdAuthRequest {
    #[validate(length(min = 8, max = 128, message = "Length should be between 3 to 128 chars"))]
    pub id: String
}

impl CustomIdAuthRequest {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

unsafe impl Send for CustomIdAuthRequest {}
unsafe impl Sync for CustomIdAuthRequest {}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Email and/or password not valid")]
    EmailOrPasswordNotValid,

    #[error("Session token could not generated")]
    TokenCouldNotGenerated,
    
    #[error("User token is not valid")]
    TokenNotValid
}

pub struct AuthManager<DB: DatabaseTrait + ?Sized> {
    config: Arc<YummyConfig>,
    database: Arc<Pool>,
    _auth: PhantomData<DB>
}

impl<DB: DatabaseTrait + ?Sized> AuthManager<DB> {
    pub fn new(config: Arc<YummyConfig>, database: Arc<Pool>) -> Self {
        Self {
            config,
            database,
            _auth: PhantomData
        }
    }

    pub fn generate_token(&self, id: UserId, name: Option<String>, email: Option<String>, session: Option<SessionId>) -> anyhow::Result<SessionToken> {
        match generate_auth(self.config.clone(), UserJwt {
            id,
            session: session.unwrap_or_else(SessionId::new) ,
            name,
            email
        }) {
            Some(token) => Ok(SessionToken(token)),
            _ => Err(anyhow::anyhow!(AuthError::TokenCouldNotGenerated))
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Actor for AuthManager<DB> {
    type Context = Context<Self>;
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<EmailAuthRequest> for AuthManager<DB> {
    type Result = anyhow::Result<SessionToken>;

    #[tracing::instrument(name="Auth::ViaEmail", skip(self, _ctx))]
    fn handle(&mut self, auth: EmailAuthRequest, _ctx: &mut Context<Self>) -> Self::Result {

        let mut connection = self.database.get()?;
        let user_info: Option<(RowId, Option<String>, String)> = DB::user_login_via_email(&mut connection, &auth.email)?;

        let (user_id, name) = match (user_info, auth.if_not_exist_create) {
            (Some((user_id, name, password)), _) => {
                if auth.password != password {
                    return Err(anyhow!(AuthError::EmailOrPasswordNotValid));
                }

                (user_id, name)
            },
            (None, true) => (DB::create_user_via_email(&mut connection, &auth.email, &auth.password)?, None),
            _ => return Err(anyhow!(AuthError::EmailOrPasswordNotValid))
        };
        
        let response = self.generate_token(UserId(user_id.0), name, Some(auth.email.to_string()), None)?;
        Ok(response)
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<DeviceIdAuthRequest> for AuthManager<DB> {
    type Result = anyhow::Result<SessionToken>;

    #[tracing::instrument(name="Auth::ViaDeviceId", skip(self, _ctx))]
    fn handle(&mut self, auth: DeviceIdAuthRequest, _ctx: &mut Context<Self>) -> Self::Result {

        let mut connection = self.database.get()?;
        let user_info: Option<(RowId, Option<String>, Option<String>)> = DB::user_login_via_device_id(&mut connection, &auth.id)?;

        let (user_id, name, email) = match user_info {
            Some((user_id, name, email)) => (user_id, name, email),
            None => (DB::create_user_via_device_id(&mut connection, &auth.id)?, None, None)
        };
        
        let response = self.generate_token(UserId(user_id.0), name, email, None)?;
        Ok(response)
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<CustomIdAuthRequest> for AuthManager<DB> {
    type Result = anyhow::Result<SessionToken>;

    #[tracing::instrument(name="Auth::ViaCustomId", skip(self, _ctx))]
    fn handle(&mut self, auth: CustomIdAuthRequest, _ctx: &mut Context<Self>) -> Self::Result {

        let mut connection = self.database.get()?;
        let user_info: Option<(RowId, Option<String>, Option<String>)> = DB::user_login_via_custom_id(&mut connection, &auth.id)?;

        let (user_id, name, email) = match user_info {
            Some((user_id, name, email)) => (user_id, name, email),
            None => (DB::create_user_via_custom_id(&mut connection, &auth.id)?, None, None)
        };
        
        let response = self.generate_token(UserId(user_id.0), name, email, None)?;
        Ok(response)
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<RefreshTokenRequest> for AuthManager<DB> {
    type Result = anyhow::Result<SessionToken>;

    #[tracing::instrument(name="Manager::Refresh", skip(self, _ctx))]
    fn handle(&mut self, token: RefreshTokenRequest, _ctx: &mut Context<Self>) -> Self::Result {
        match validate_auth(self.config.clone(), token.token) {
            Some(claims) => self.generate_token(claims.user.id, claims.user.name, claims.user.email, Some(claims.user.session)),
            None => Err(anyhow!(AuthError::TokenNotValid))
        }
    }
}

#[cfg(test)]
mod tests {
    use general::config::YummyConfig;
    use general::config::get_configuration;
    use general::auth::validate_auth;
    use general::model::SessionToken;
    use std::sync::Arc;

    use actix::Actor;
    use actix::Addr;
    use anyhow::Ok;
    use database::{create_database, create_connection};
    
    use super::AuthManager;
    use super::DeviceIdAuthRequest;
    use super::EmailAuthRequest;
    use super::RefreshTokenRequest;
    use super::CustomIdAuthRequest;

    fn create_actor() -> anyhow::Result<(Addr<AuthManager<database::SqliteStore>>, Arc<YummyConfig>)> {
        let config = get_configuration();
        let connection = create_connection(":memory:")?;
        create_database(&mut connection.clone().get()?)?;
        Ok((AuthManager::<database::SqliteStore>::new(config.clone(), Arc::new(connection)).start(), config))
    }

    /* email unit tests */
    #[actix::test]
    async fn create_user_via_email() -> anyhow::Result<()> {
        let (address, _) = create_actor()?;
        address.send(EmailAuthRequest {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: true
        }).await??;
        Ok(())
    }

    #[actix::test]
    async fn login_user_via_email() -> anyhow::Result<()> {
        let (address, _) = create_actor()?;
        address.send(EmailAuthRequest {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: true
        }).await??;

        address.send(EmailAuthRequest {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: false
        }).await??;

        Ok(())
    }

    #[actix::test]
    async fn failed_login_user_via_email_1() -> anyhow::Result<()> {
        let (address, _) = create_actor()?;
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
        let (address, _) = create_actor()?;
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
        let (address, _) = create_actor()?;
        address.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
        Ok(())
    }

    #[actix::test]
    async fn login_user_via_device_id() -> anyhow::Result<()> {
        let (address, _) = create_actor()?;
        let created_token = address.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
        let logged_in_token = address.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
        assert_ne!(created_token, logged_in_token);

        Ok(())
    }

    #[actix::test]
    async fn login_users_via_device_id() -> anyhow::Result<()> {
        let (address, _) = create_actor()?;
        let login_1 = address.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
        let login_2 = address.send(DeviceIdAuthRequest::new("abcdef".to_string())).await??;
        assert_ne!(login_1, login_2);

        Ok(())
    }

    /* custom id unit tests */
    #[actix::test]
    async fn create_user_via_custom_id() -> anyhow::Result<()> {
        let (address, _) = create_actor()?;
        address.send(CustomIdAuthRequest::new("1234567890".to_string())).await??;
        Ok(())
    }

    #[actix::test]
    async fn login_user_via_custom_id() -> anyhow::Result<()> {
        let (address, _) = create_actor()?;
        let created_token = address.send(CustomIdAuthRequest::new("1234567890".to_string())).await??;
        let logged_in_token = address.send(CustomIdAuthRequest::new("1234567890".to_string())).await??;
        assert_ne!(created_token, logged_in_token);

        Ok(())
    }

    #[actix::test]
    async fn login_users_via_custom_id() -> anyhow::Result<()> {
        let (address, _) = create_actor()?;
        let login_1 = address.send(CustomIdAuthRequest::new("1234567890".to_string())).await??;
        let login_2 = address.send(CustomIdAuthRequest::new("abcdef".to_string())).await??;
        assert_ne!(login_1, login_2);

        Ok(())
    }

    /* refreh token unit tests */
    #[actix::test]
    async fn token_refresh_test_1() -> anyhow::Result<()> {
        let (address, config) = create_actor()?;
        let old_token: SessionToken = address.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;

        // Wait 1 second
        actix::clock::sleep(std::time::Duration::new(1, 0)).await;
        let new_token: SessionToken = address.send(RefreshTokenRequest { token: old_token.0.to_string() }).await??;
       
        assert_ne!(old_token.clone(), new_token.clone());

        let old_claims =  validate_auth(config.clone(), old_token.0).unwrap();
        let new_claims =  validate_auth(config.clone(), new_token.0).unwrap();

        assert_eq!(old_claims.user.id.clone(), new_claims.user.id.clone());
        assert_eq!(old_claims.user.name.clone(), new_claims.user.name.clone());
        assert_eq!(old_claims.user.email.clone(), new_claims.user.email.clone());
        assert_eq!(old_claims.user.session.clone(), new_claims.user.session.clone());

        assert!(old_claims.exp < new_claims.exp);

        Ok(())
    }

    #[actix::test]
    async fn token_refresh_test_2() -> anyhow::Result<()> {
        let (address, config) = create_actor()?;
        let old_token: SessionToken = address.send(EmailAuthRequest {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: true
        }).await??;

        // Wait 1 second
        actix::clock::sleep(std::time::Duration::new(1, 0)).await;
        let new_token: SessionToken = address.send(RefreshTokenRequest{ token: old_token.0.to_string() }).await??;
       
        assert_ne!(old_token.clone(), new_token.clone());

        let old_claims =  validate_auth(config.clone(), old_token.0).unwrap();
        let new_claims =  validate_auth(config.clone(), new_token.0).unwrap();

        assert_eq!(old_claims.user.id.clone(), new_claims.user.id.clone());
        assert_eq!(old_claims.user.name.clone(), new_claims.user.name.clone());
        assert_eq!(old_claims.user.email.clone(), new_claims.user.email.clone());
        assert_eq!(old_claims.user.session.clone(), new_claims.user.session.clone());

        assert!(old_claims.exp < new_claims.exp);

        Ok(())
    }
}