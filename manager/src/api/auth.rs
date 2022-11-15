use general::auth::{generate_auth, UserJwt, validate_auth};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use general::config::YummyConfig;

use actix::{Context, Handler, Actor};
use actix::prelude::Message;
use database::{RowId, Pool};
use database::auth::{AuthStore, AuthStoreTrait};
use thiserror::Error;
use anyhow::anyhow;
use validator::Validate;

use general::model::{SessionToken, UserId, SessionId};

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<SessionToken>")]
pub struct EmailAuth {
    #[validate(email(message="Email address is not valid"))]
    pub email: String,

    #[validate(length(min = 3, max = 32, message = "Length should be between 3 to 32 chars"))]
    pub password: String,

    pub if_not_exist_create: bool,
}

unsafe impl Send for EmailAuth {}
unsafe impl Sync for EmailAuth {}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<SessionToken>")]
pub struct RefreshToken {

    #[validate(length(min = 275, max = 1024, message = "Length should be between 275 to 1024 chars"))]
    pub token: String
}

impl From<SessionToken> for RefreshToken {
    fn from(token: SessionToken) -> Self {
        RefreshToken { token: token.0 }
    }
}

unsafe impl Send for RefreshToken {}
unsafe impl Sync for RefreshToken {}

#[derive(Message, Debug, Validate)]
#[rtype(result = "anyhow::Result<SessionToken>")]
pub struct DeviceIdAuth {
    #[validate(length(min = 8, max = 128, message = "Length should be between 8 to 128 chars"))]
    pub id: String
}

impl DeviceIdAuth {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

unsafe impl Send for DeviceIdAuth {}
unsafe impl Sync for DeviceIdAuth {}

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

    pub fn generate_token(&self, id: UserId, name: Option<String>, email: Option<String>, session: Option<SessionId>) -> anyhow::Result<SessionToken> {
        match generate_auth(self.config.clone(), UserJwt {
            id,
            session: session.unwrap_or_else(|| SessionId::new()) ,
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
        let user_info: Option<(RowId, Option<String>, String)> = auth_store.user_login_via_email(&auth.email)?;

        let (user_id, name) = match (user_info, auth.if_not_exist_create) {
            (Some((user_id, name, password)), _) => {
                if auth.password != password {
                    return Err(anyhow!(AuthError::EmailOrPasswordNotValid));
                }

                (user_id, name)
            },
            (None, true) => (auth_store.create_user_via_email(&auth.email, &auth.password)?, None),
            _ => return Err(anyhow!(AuthError::EmailOrPasswordNotValid))
        };
        
        let response = self.generate_token(UserId(user_id.0), name, Some(auth.email.to_string()), None)?;
        Ok(response)
    }
}

impl<A: AuthStoreTrait + std::marker::Unpin + 'static> Handler<DeviceIdAuth> for AuthManager<A> {
    type Result = anyhow::Result<SessionToken>;

    #[tracing::instrument(name="Auth::ViaEmail", skip(self, _ctx))]
    fn handle(&mut self, auth: DeviceIdAuth, _ctx: &mut Context<Self>) -> Self::Result {

        let mut auth_store = AuthStore::new(self.database.get()?);
        let user_info: Option<(RowId, Option<String>, Option<String>)> = auth_store.user_login_via_device_id(&auth.id)?;

        let (user_id, name, email) = match user_info {
            Some((user_id, name, email)) => (user_id, name, email),
            None => (auth_store.create_user_via_device_id(&auth.id)?, None, None)
        };
        
        let response = self.generate_token(UserId(user_id.0), name, email, None)?;
        Ok(response)
    }
}

impl<A: AuthStoreTrait + std::marker::Unpin + 'static> Handler<RefreshToken> for AuthManager<A> {
    type Result = anyhow::Result<SessionToken>;

    #[tracing::instrument(name="Manager::Refresh", skip(self, _ctx))]
    fn handle(&mut self, token: RefreshToken, _ctx: &mut Context<Self>) -> Self::Result {
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
    use super::DeviceIdAuth;
    use super::EmailAuth;
    use super::RefreshToken;

    fn create_actor() -> anyhow::Result<(Addr<AuthManager<database::auth::AuthStore>>, Arc<YummyConfig>)> {
        let config = get_configuration();
        let connection = create_connection(":memory:")?;
        create_database(&mut connection.clone().get()?)?;
        Ok((AuthManager::<database::auth::AuthStore>::new(config.clone(), Arc::new(connection)).start(), config))
    }

    #[actix::test]
    async fn create_user_via_email() -> anyhow::Result<()> {
        let (address, _) = create_actor()?;
        address.send(EmailAuth {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: true
        }).await??;
        Ok(())
    }

    #[actix::test]
    async fn login_user_via_email() -> anyhow::Result<()> {
        let (address, _) = create_actor()?;
        address.send(EmailAuth {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: true
        }).await??;

        address.send(EmailAuth {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: false
        }).await??;

        Ok(())
    }

    #[actix::test]
    async fn failed_login_user_via_email_1() -> anyhow::Result<()> {
        let (address, _) = create_actor()?;
        let result = address.send(EmailAuth {
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
        address.send(EmailAuth {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: true
        }).await??;

        let result = address.send(EmailAuth {
            email: "erhanbaris@gmail.com".to_string(),
            password: "wrong password".to_string(),
            if_not_exist_create: true
        }).await?;

        assert_eq!(result.unwrap_err().to_string(), "Email and/or password not valid".to_string());
        Ok(())
    }

    #[actix::test]
    async fn create_user_via_device_id() -> anyhow::Result<()> {
        let (address, _) = create_actor()?;
        address.send(DeviceIdAuth::new("1234567890".to_string())).await??;
        Ok(())
    }

    #[actix::test]
    async fn login_user_via_device_id() -> anyhow::Result<()> {
        let (address, _) = create_actor()?;
        let created_token = address.send(DeviceIdAuth::new("1234567890".to_string())).await??;
        let logged_in_token = address.send(DeviceIdAuth::new("1234567890".to_string())).await??;
        assert_ne!(created_token, logged_in_token);

        Ok(())
    }

    #[actix::test]
    async fn login_users_via_device_id() -> anyhow::Result<()> {
        let (address, _) = create_actor()?;
        let login_1 = address.send(DeviceIdAuth::new("1234567890".to_string())).await??;
        let login_2 = address.send(DeviceIdAuth::new("abcdef".to_string())).await??;
        assert_ne!(login_1, login_2);

        Ok(())
    }

    #[actix::test]
    async fn token_refresh_test_1() -> anyhow::Result<()> {
        let (address, config) = create_actor()?;
        let old_token: SessionToken = address.send(DeviceIdAuth::new("1234567890".to_string())).await??;

        // Wait 1 second
        actix::clock::sleep(std::time::Duration::new(1, 0)).await;
        let new_token: SessionToken = address.send(RefreshToken { token: old_token.0.to_string() }).await??;
       
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
        let old_token: SessionToken = address.send(EmailAuth {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: true
        }).await??;

        // Wait 1 second
        actix::clock::sleep(std::time::Duration::new(1, 0)).await;
        let new_token: SessionToken = address.send(RefreshToken{ token: old_token.0.to_string() }).await??;
       
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