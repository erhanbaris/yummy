use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use database::model::PrivateUserModel;
use general::config::YummyConfig;

use actix::{Context, Actor, Handler};
use actix::prelude::Message;
use database::{Pool, DatabaseTrait};
use validator::Validate;

use general::model::UserId;

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<Option<PrivateUserModel>>")]
pub struct GetUser {
    pub user: UserId
}

pub struct UserManager<DB: DatabaseTrait + ?Sized> {
    config: Arc<YummyConfig>,
    database: Arc<Pool>,
    _marker: PhantomData<DB>
}

impl<DB: DatabaseTrait + ?Sized> UserManager<DB> {
    pub fn new(config: Arc<YummyConfig>, database: Arc<Pool>) -> Self {
        Self {
            config,
            database,
            _marker: PhantomData
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Actor for UserManager<DB> {
    type Context = Context<Self>;
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<GetUser> for UserManager<DB> {
    type Result = anyhow::Result<Option<PrivateUserModel>>;

    #[tracing::instrument(name="User::GetUser", skip(self, _ctx))]
    fn handle(&mut self, model: GetUser, _ctx: &mut Context<Self>) -> Self::Result {
        DB::get_user(&mut self.database.get()?, model.user.0.into())
    }
}

#[cfg(test)]
mod tests {
    use general::auth::validate_auth;
    use general::config::YummyConfig;
    use general::config::get_configuration;
    use general::model::UserId;
    use std::sync::Arc;

    use actix::Actor;
    use actix::Addr;
    use anyhow::Ok;
    use database::{create_database, create_connection};

    use super::GetUser;
    use super::UserManager;
    use crate::api::auth::AuthManager;
    use crate::api::auth::DeviceIdAuth;

    fn create_actor() -> anyhow::Result<(Addr<UserManager<database::SqliteStore>>, Addr<AuthManager<database::SqliteStore>>, Arc<YummyConfig>)> {
        let config = get_configuration();
        let connection = create_connection(":memory:")?;
        create_database(&mut connection.clone().get()?)?;
        Ok((UserManager::<database::SqliteStore>::new(config.clone(), Arc::new(connection.clone())).start(), AuthManager::<database::SqliteStore>::new(config.clone(), Arc::new(connection)).start(), config))
    }
    
    #[actix::test]
    async fn get_user_1() -> anyhow::Result<()> {
        let (user_manager, _, _) = create_actor()?;
        let user = user_manager.send(GetUser {
            user: UserId::default()
        }).await??;
        assert!(user.is_none());
        Ok(())
    }

    #[actix::test]
    async fn get_user_2() -> anyhow::Result<()> {
        let (user_manager, auth_manager, config) = create_actor()?;

        let token = auth_manager.send(DeviceIdAuth::new("1234567890".to_string())).await??;
        let user = validate_auth(config, token.0).unwrap();
        let user = user_manager.send(GetUser {
            user: user.user.id
        }).await??;
        assert!(user.is_some());
        Ok(())
    }
}