pub mod model;

use std::time::Duration;
use std::{marker::PhantomData, ops::Deref};
use std::sync::Arc;
use database::model::{UserUpdate, PublicUserModel, PrivateUserModel};

use actix::{Context, Actor, Handler};
use database::{Pool, DatabaseTrait, RowId};

use moka::sync::Cache;
use uuid::Uuid;

use crate::response::Response;
use crate::api::auth::model::AuthError;

use self::model::*;

pub struct UserManager<DB: DatabaseTrait + ?Sized> {
    database: Arc<Pool>,
    _marker: PhantomData<DB>,

    // Caches
    cache_public_user_info: Cache<Uuid, PublicUserModel>,
    cache_private_user_info: Cache<Uuid, PrivateUserModel>
}

impl<DB: DatabaseTrait + ?Sized> UserManager<DB> {
    pub fn new(database: Arc<Pool>) -> Self {
        Self {
            database,
            _marker: PhantomData,
            cache_public_user_info: Cache::builder().time_to_idle(Duration::from_secs(5*60)).build(),
            cache_private_user_info: Cache::builder().time_to_idle(Duration::from_secs(5*60)).build()
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Actor for UserManager<DB> {
    type Context = Context<Self>;
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static>  UserManager<DB> {
    fn cleanup_user_cache(&self, user_id: &Uuid) {
        self.cache_public_user_info.invalidate(user_id);
        self.cache_private_user_info.invalidate(user_id);
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<GetDetailedUserInfo> for UserManager<DB> {
    type Result = anyhow::Result<Response>;

    #[tracing::instrument(name="User::GetPrivateUser", skip(self, _ctx))]
    fn handle(&mut self, model: GetDetailedUserInfo, _ctx: &mut Context<Self>) -> Self::Result {

        let user_id = match model.user.deref() {
            Some(user) => user.user.0,
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        };

        let user = DB::get_private_user_info(&mut self.database.get()?, user_id.into())?;

        match user {
            Some(user) => Ok(Response::UserPrivateInfo(user)),
            None => Err(anyhow::anyhow!(UserError::UserNotFound))
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<GetPublicUserInfo> for UserManager<DB> {
    type Result = anyhow::Result<Response>;

    #[tracing::instrument(name="User::GetPublicUser", skip(self, _ctx))]
    fn handle(&mut self, model: GetPublicUserInfo, _ctx: &mut Context<Self>) -> Self::Result {
        let user_id = model.target_user.0.into();

        match self.cache_public_user_info.get(&user_id) {
            Some(user) => Ok(Response::UserPublicInfo(user)),
            None => {
                let user = DB::get_public_user_info(&mut self.database.get()?, model.target_user.0.into())?;

                match user {
                    Some(user) => {
                        self.cache_public_user_info.insert(user_id, user.clone());
                        Ok(Response::UserPublicInfo(user))
                    },
                    None => Err(anyhow::anyhow!(UserError::UserNotFound))
                }
            }
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<UpdateUser> for UserManager<DB> {
    type Result = anyhow::Result<Response>;

    #[tracing::instrument(name="User::UpdateUser", skip(self, _ctx))]
    fn handle(&mut self, model: UpdateUser, _ctx: &mut Context<Self>) -> Self::Result {

        let original_user_id = match model.user.deref() {
            Some(user) => user.user.0,
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        };

        if model.custom_id.is_none() && model.device_id.is_none() && model.email.is_none() && model.name.is_none() && model.password.is_none() {
            return Err(anyhow::anyhow!(UserError::UpdateInformationMissing));
        }

        let mut updates = UserUpdate::default();
        let user_id = RowId(original_user_id);

        let mut connection = self.database.get()?;
        let user = match DB::get_private_user_info(&mut connection, user_id)? {
            Some(user) => user,
            None => return Err(anyhow::anyhow!(UserError::UserNotFound))
        };

        updates.custom_id = model.custom_id.map(|item| match item.trim().len() == 0 { true => None, false => Some(item)} );
        updates.device_id = model.device_id.map(|item| match item.trim().len() == 0 { true => None, false => Some(item)} );
        updates.name = model.name.map(|item| match item.trim().len() == 0 { true => None, false => Some(item)} );

        if let Some(password) = model.password {
            if password.trim().len() < 4 {
                return Err(anyhow::anyhow!(UserError::PasswordIsTooSmall))
            }
            updates.password = Some(password);
        }

        if let Some(email) = model.email {
            if user.email.is_some() {
                return Err(anyhow::anyhow!(UserError::CannotChangeEmail));
            }
            updates.email = Some(email)
        }

        match DB::update_user(&mut connection, user_id, updates)? {
            0 => Err(anyhow::anyhow!(UserError::UserNotFound)),
            _ => {
                self.cleanup_user_cache(&original_user_id);
                Ok(Response::None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use general::auth::UserAuth;
    use general::auth::validate_auth;
    use general::config::YummyConfig;
    use general::config::get_configuration;
    use std::sync::Arc;

    use actix::Actor;
    use actix::Addr;
    use anyhow::Ok;
    use database::{create_database, create_connection};

    use super::*;
    use crate::api::auth::AuthManager;
    use crate::api::auth::model::*;

    fn create_actor() -> anyhow::Result<(Addr<UserManager<database::SqliteStore>>, Addr<AuthManager<database::SqliteStore>>, Arc<YummyConfig>)> {
        let config = get_configuration();
        let connection = create_connection(":memory:")?;
        create_database(&mut connection.clone().get()?)?;
        Ok((UserManager::<database::SqliteStore>::new(Arc::new(connection.clone())).start(), AuthManager::<database::SqliteStore>::new(config.clone(), Arc::new(connection)).start(), config))
    }
    
    #[actix::test]
    async fn get_private_user_1() -> anyhow::Result<()> {
        let (user_manager, _, _) = create_actor()?;
        let user = user_manager.send(GetDetailedUserInfo {
            user: Arc::new(None)
        }).await?;
        assert!(user.is_err());
        Ok(())
    }

    #[actix::test]
    async fn get_private_user_2() -> anyhow::Result<()> {
        let (user_manager, auth_manager, config) = create_actor()?;

        let token = auth_manager.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
        let token = match token {
            Response::Auth(token, _) => token,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
        };

        let user = validate_auth(config, token).unwrap();
        let user = user_manager.send(GetDetailedUserInfo {
            user: Arc::new(Some(UserAuth {
                user: user.user.id,
                session: user.user.session
            }))
        }).await??;

        let user = match user {
            Response::UserPrivateInfo(model) => model,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::UserPrivateInfo'")); }
        };

        assert_eq!(user.device_id, Some("1234567890".to_string()));
        Ok(())
    }

    #[actix::test]
    async fn fail_update_get_user_1() -> anyhow::Result<()> {
        let (user_manager, _, _) = create_actor()?;
        let result = user_manager.send(UpdateUser {
            user: Arc::new(None),
            ..Default::default()
        }).await?;
        assert!(result.is_err());
        Ok(())
    }

    #[actix::test]
    async fn fail_update_get_user_2() -> anyhow::Result<()> {
        let (user_manager, auth_manager, config) = create_actor()?;

        let token = auth_manager.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
        let token = match token {
            Response::Auth(token, _) => token,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
        };

        let user = validate_auth(config, token).unwrap();
        let result = user_manager.send(UpdateUser {
            user: Arc::new(Some(UserAuth {
                user: user.user.id,
                session: user.user.session
            })),
            ..Default::default()
        }).await?;
        assert!(result.is_err());
        Ok(())
    }

    #[actix::test]
    async fn fail_update_get_user_3() -> anyhow::Result<()> {
        let (user_manager, auth_manager, config) = create_actor()?;

        let token = auth_manager.send(EmailAuthRequest {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: true
        }).await??;
        
        let token = match token {
            Response::Auth(token, _) => token,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
        };

        let user = validate_auth(config, token).unwrap();
        let result = user_manager.send(UpdateUser {
            user: Arc::new(Some(UserAuth {
                user: user.user.id,
                session: user.user.session
            })),
            email: Some("erhanbaris@gmail.com".to_string()),
            ..Default::default()
        }).await?;

        match result {
            std::result::Result::Ok(_) => { assert!(false, "Expected 'CannotChangeEmail' error message"); },
            Err(error) => { assert_eq!(error.downcast_ref::<UserError>().unwrap(), &UserError::CannotChangeEmail); }
        };

        Ok(())
    }

    #[actix::test]
    async fn fail_update_get_user_4() -> anyhow::Result<()> {
        let (user_manager, auth_manager, config) = create_actor()?;

        let token = auth_manager.send(EmailAuthRequest {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: true
        }).await??;
        
        let token = match token {
            Response::Auth(token, _) => token,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
        };

        let user = validate_auth(config, token).unwrap();
        let result = user_manager.send(UpdateUser {
            user: Arc::new(Some(UserAuth {
                user: user.user.id,
                session: user.user.session
            })),
            ..Default::default()
        }).await?;

        match result {
            std::result::Result::Ok(_) => { assert!(false, "Expected 'UpdateInformationMissing' error message"); },
            Err(error) => { assert_eq!(error.downcast_ref::<UserError>().unwrap(), &UserError::UpdateInformationMissing); }
        };

        Ok(())
    }

    #[actix::test]
    async fn fail_update_password() -> anyhow::Result<()> {
        let (user_manager, auth_manager, config) = create_actor()?;

        let token = auth_manager.send(EmailAuthRequest {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: true
        }).await??;

        let token = match token {
            Response::Auth(token, _) => token,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
        };

        let user_jwt = validate_auth(config, token).unwrap().user;
        let user_auth = Arc::new(Some(UserAuth {
            user: user_jwt.id,
            session: user_jwt.session
        }));
        
        let result = user_manager.send(UpdateUser {
            user: user_auth.clone(),
            password: Some("123".to_string()),
            ..Default::default()
        }).await?;

        match result {
            std::result::Result::Ok(_) => { assert!(false, "Expected 'PasswordIsTooSmall' error message"); },
            Err(error) => { assert_eq!(error.downcast_ref::<UserError>().unwrap(), &UserError::PasswordIsTooSmall); }
        };

        Ok(())
    }

    #[actix::test]
    async fn fail_update_email() -> anyhow::Result<()> {
        let (user_manager, auth_manager, config) = create_actor()?;

        let token = auth_manager.send(EmailAuthRequest {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: true
        }).await??;

        let token = match token {
            Response::Auth(token, _) => token,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
        };

        let user_jwt = validate_auth(config, token).unwrap().user;
        let user_auth = Arc::new(Some(UserAuth {
            user: user_jwt.id,
            session: user_jwt.session
        }));

        let result = user_manager.send(UpdateUser {
            user: user_auth.clone(),
            email: Some("erhanbaris@gmail.com".to_string()),
            ..Default::default()
        }).await?;

        match result {
            std::result::Result::Ok(_) => { assert!(false, "Expected 'CannotChangeEmail' error message"); },
            Err(error) => { assert_eq!(error.downcast_ref::<UserError>().unwrap(), &UserError::CannotChangeEmail); }
        };

        Ok(())
    }

    #[actix::test]
    async fn update_get_public_user_1() -> anyhow::Result<()> {
        let (user_manager, auth_manager, config) = create_actor()?;

        let token = auth_manager.send(EmailAuthRequest {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: true
        }).await??;

        let token = match token {
            Response::Auth(token, _) => token,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
        };

        let user_jwt = validate_auth(config, token).unwrap().user;
        let user_auth = Arc::new(Some(UserAuth {
            user: user_jwt.id,
            session: user_jwt.session
        }));

        let user = user_manager.send(GetPublicUserInfo {
            user: user_auth.clone(),
            target_user: user_jwt.id
        }).await??;

        let user = match user {
            Response::UserPublicInfo(model) => model,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::UserPublicInfo'")); }
        };
        
        assert_eq!(user.name, None);
        
        user_manager.send(UpdateUser {
            user: user_auth.clone(),
            name: Some("Erhan".to_string()),
            ..Default::default()
        }).await??;

        let user = user_manager.send(GetPublicUserInfo {
            user: user_auth.clone(),
            target_user: user_jwt.id
        }).await??;

        let user = match user {
            Response::UserPublicInfo(model) => model,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::UserPublicInfo'")); }
        };

        assert_eq!(user.name, Some("Erhan".to_string()));

        Ok(())
    }

    #[actix::test]
    async fn update_get_public_user_2() -> anyhow::Result<()> {
        let (user_manager, auth_manager, config) = create_actor()?;

        let token = auth_manager.send(EmailAuthRequest {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: true
        }).await??;

        let token = match token {
            Response::Auth(token, _) => token,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
        };

        let user_jwt = validate_auth(config, token).unwrap().user;
        let user_auth = Arc::new(Some(UserAuth {
            user: user_jwt.id,
            session: user_jwt.session
        }));

        let user = user_manager.send(GetPublicUserInfo {
            user: user_auth.clone(),
            target_user: user_jwt.id
        }).await??;

        let user = match user {
            Response::UserPublicInfo(model) => model,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::UserPublicInfo'")); }
        };
        
        assert_eq!(user.name, None);
        
        user_manager.send(UpdateUser {
            user: user_auth.clone(),
            name: Some("Erhan".to_string()),
            ..Default::default()
        }).await??;

        let user = user_manager.send(GetPublicUserInfo {
            user: user_auth.clone(),
            target_user: user_jwt.id
        }).await??;

        let user = match user {
            Response::UserPublicInfo(model) => model,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::UserPublicInfo'")); }
        };

        assert_eq!(user.name, Some("Erhan".to_string()));

        /* Cleanup fields */
        user_manager.send(UpdateUser {
            user: user_auth.clone(),
            name: Some("Erhan".to_string()),
            ..Default::default()
        }).await??;

        let user = user_manager.send(GetPublicUserInfo {
            user: user_auth.clone(),
            target_user: user_jwt.id
        }).await??;

        let user = match user {
            Response::UserPublicInfo(model) => model,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::UserPublicInfo'")); }
        };

        assert_eq!(user.name, Some("Erhan".to_string()));

        Ok(())
    }

    #[actix::test]
    async fn update_get_private_user_1() -> anyhow::Result<()> {
        let (user_manager, auth_manager, config) = create_actor()?;

        let token = auth_manager.send(EmailAuthRequest {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: true
        }).await??;

        let token = match token {
            Response::Auth(token, _) => token,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
        };

        let user_jwt = validate_auth(config, token).unwrap().user;
        let user_auth = Arc::new(Some(UserAuth {
            user: user_jwt.id,
            session: user_jwt.session
        }));

        let user = user_manager.send(GetDetailedUserInfo {
            user: user_auth.clone()
        }).await??;

        let user = match user {
            Response::UserPrivateInfo(model) => model,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::UserPrivateInfo'")); }
        };
        
        assert_eq!(user.name, None);
        assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));
        
        user_manager.send(UpdateUser {
            user: user_auth.clone(),
            name: Some("Erhan".to_string()),
            ..Default::default()
        }).await??;

        let user = user_manager.send(GetDetailedUserInfo {
            user: user_auth.clone()
        }).await??;

        let user = match user {
            Response::UserPrivateInfo(model) => model,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::UserPrivateInfo'")); }
        };

        assert_eq!(user.name, Some("Erhan".to_string()));
        assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));

        Ok(())
    }

    #[actix::test]
    async fn update_get_private_user_2() -> anyhow::Result<()> {
        let (user_manager, auth_manager, config) = create_actor()?;

        let token = auth_manager.send(EmailAuthRequest {
            email: "erhanbaris@gmail.com".to_string(),
            password: "erhan".to_string(),
            if_not_exist_create: true
        }).await??;

        let token = match token {
            Response::Auth(token, _) => token,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
        };

        let user_jwt = validate_auth(config, token).unwrap().user;
        let user_auth = Arc::new(Some(UserAuth {
            user: user_jwt.id,
            session: user_jwt.session
        }));

        let user = user_manager.send(GetDetailedUserInfo {
            user: user_auth.clone()
        }).await??;

        let user = match user {
            Response::UserPrivateInfo(model) => model,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::UserPrivateInfo'")); }
        };
        
        assert_eq!(user.name, None);
        assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));
        
        user_manager.send(UpdateUser {
            user: user_auth.clone(),
            name: Some("Erhan".to_string()),
            ..Default::default()
        }).await??;

        let user = user_manager.send(GetDetailedUserInfo {
            user: user_auth.clone()
        }).await??;

        let user = match user {
            Response::UserPrivateInfo(model) => model,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::UserPrivateInfo'")); }
        };

        assert_eq!(user.name, Some("Erhan".to_string()));
        assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));

        /* Cleanup fields */
        user_manager.send(UpdateUser {
            user: user_auth.clone(),
            name: Some("Erhan".to_string()),
            ..Default::default()
        }).await??;

        let user = user_manager.send(GetDetailedUserInfo {
            user: user_auth.clone()
        }).await??;

        let user = match user {
            Response::UserPrivateInfo(model) => model,
            _ => { return Err(anyhow::anyhow!("Expected 'Response::UserPrivateInfo'")); }
        };

        assert_eq!(user.name, Some("Erhan".to_string()));
        assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));

        Ok(())
    }
}