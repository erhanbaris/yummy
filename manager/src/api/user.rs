use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use database::model::{PrivateUserModel, PublicUserModel, UserUpdate};
use general::config::YummyConfig;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use actix::{Context, Actor, Handler};
use actix::prelude::Message;
use database::{Pool, DatabaseTrait, RowId};
use validator::Validate;

use general::model::UserId;

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<PrivateUserModel>")]
pub struct GetDetailedUserInfo {
    pub user: UserId
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<PublicUserModel>")]
pub struct GetPublicUserInfo {
    pub user: UserId
}

#[derive(Message, Validate, Debug)]
#[rtype(result = "anyhow::Result<PublicUserModel>")]
pub struct GetMe {
    pub user: UserId
}

#[derive(Message, Validate, Debug, Default)]
#[rtype(result = "anyhow::Result<()>")]
pub struct UpdateUser {
    pub user: UserId,
    pub name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub password: Option<String>,
    pub device_id: Option<String>,
    pub custom_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UpdateUserFieldType {
    #[serde(rename = "name")]
    Name,

    #[serde(rename = "password")]
    Password,

    #[serde(rename = "device_id")]
    DeviceId,

    #[serde(rename = "custom_id")]
    CustomId,

    #[serde(rename = "email")]
    Email
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserField {
    #[serde(rename = "type")]
    field: UpdateUserFieldType,
    value: Option<String>
}

#[derive(Error, Debug, PartialEq)]
pub enum UserError {
    #[error("User not found")]
    UserNotFound,

    #[error("The user's email address cannot be changed.")]
    CannotChangeEmail,

    #[error("The password is too small")]
    PasswordIsTooSmall,

    #[error("Update information missing")]
    UpdateInformationMissing
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

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<GetDetailedUserInfo> for UserManager<DB> {
    type Result = anyhow::Result<PrivateUserModel>;

    #[tracing::instrument(name="User::GetPrivateUser", skip(self, _ctx))]
    fn handle(&mut self, model: GetDetailedUserInfo, _ctx: &mut Context<Self>) -> Self::Result {
        let user = DB::get_private_user_info(&mut self.database.get()?, model.user.0.into())?;

        match user {
            Some(user) => Ok(user),
            None => Err(anyhow::anyhow!(UserError::UserNotFound))
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<GetPublicUserInfo> for UserManager<DB> {
    type Result = anyhow::Result<PublicUserModel>;

    #[tracing::instrument(name="User::GetPublicUser", skip(self, _ctx))]
    fn handle(&mut self, model: GetPublicUserInfo, _ctx: &mut Context<Self>) -> Self::Result {
        let user = DB::get_public_user_info(&mut self.database.get()?, model.user.0.into())?;

        match user {
            Some(user) => Ok(user),
            None => Err(anyhow::anyhow!(UserError::UserNotFound))
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<UpdateUser> for UserManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="User::UpdateUser", skip(self, _ctx))]
    fn handle(&mut self, model: UpdateUser, _ctx: &mut Context<Self>) -> Self::Result {

        if model.custom_id.is_none() && model.device_id.is_none() && model.email.is_none() && model.name.is_none() && model.password.is_none() {
            return Err(anyhow::anyhow!(UserError::UpdateInformationMissing));
        }

        let mut updates = UserUpdate::default();
        let user_id = RowId(model.user.0);

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
            _ => Ok(())
        }
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

    use super::*;
    use crate::api::auth::AuthManager;
    use crate::api::auth::DeviceIdAuthRequest;
    use crate::api::auth::EmailAuthRequest;

    fn create_actor() -> anyhow::Result<(Addr<UserManager<database::SqliteStore>>, Addr<AuthManager<database::SqliteStore>>, Arc<YummyConfig>)> {
        let config = get_configuration();
        let connection = create_connection(":memory:")?;
        create_database(&mut connection.clone().get()?)?;
        Ok((UserManager::<database::SqliteStore>::new(config.clone(), Arc::new(connection.clone())).start(), AuthManager::<database::SqliteStore>::new(config.clone(), Arc::new(connection)).start(), config))
    }
    
    #[actix::test]
    async fn get_private_user_1() -> anyhow::Result<()> {
        let (user_manager, _, _) = create_actor()?;
        let user = user_manager.send(GetDetailedUserInfo {
            user: UserId::default()
        }).await?;
        assert!(user.is_err());
        Ok(())
    }

    #[actix::test]
    async fn get_private_user_2() -> anyhow::Result<()> {
        let (user_manager, auth_manager, config) = create_actor()?;

        let token = auth_manager.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
        let user = validate_auth(config, token.0).unwrap();
        let user = user_manager.send(GetDetailedUserInfo {
            user: user.user.id
        }).await??;
        assert_eq!(user.device_id, Some("1234567890".to_string()));
        Ok(())
    }

    #[actix::test]
    async fn fail_update_get_user_1() -> anyhow::Result<()> {
        let (user_manager, _, _) = create_actor()?;
        let result = user_manager.send(UpdateUser {
            user: UserId::default(),
            ..Default::default()
        }).await?;
        assert!(result.is_err());
        Ok(())
    }

    #[actix::test]
    async fn fail_update_get_user_2() -> anyhow::Result<()> {
        let (user_manager, auth_manager, config) = create_actor()?;

        let token = auth_manager.send(DeviceIdAuthRequest::new("1234567890".to_string())).await??;
        let user = validate_auth(config, token.0).unwrap();
        let result = user_manager.send(UpdateUser {
            user: user.user.id,
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
        let user = validate_auth(config, token.0).unwrap();
        let result = user_manager.send(UpdateUser {
            user: user.user.id,
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
        let user = validate_auth(config, token.0).unwrap();
        let result = user_manager.send(UpdateUser {
            user: user.user.id,
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
        let user_id = validate_auth(config, token.0).unwrap().user.id;

        
        let result = user_manager.send(UpdateUser {
            user: user_id,
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
        let user_id = validate_auth(config, token.0).unwrap().user.id;

        
        let result = user_manager.send(UpdateUser {
            user: user_id,
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
        let user_id = validate_auth(config, token.0).unwrap().user.id;

        let user = user_manager.send(GetPublicUserInfo {
            user: user_id
        }).await??;
        
        assert_eq!(user.name, None);
        
        user_manager.send(UpdateUser {
            user: user_id,
            name: Some("Erhan".to_string()),
            ..Default::default()
        }).await??;

        let user = user_manager.send(GetPublicUserInfo {
            user: user_id
        }).await??;

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
        let user_id = validate_auth(config, token.0).unwrap().user.id;

        let user = user_manager.send(GetPublicUserInfo {
            user: user_id
        }).await??;
        
        assert_eq!(user.name, None);
        
        user_manager.send(UpdateUser {
            user: user_id,
            name: Some("Erhan".to_string()),
            ..Default::default()
        }).await??;

        let user = user_manager.send(GetPublicUserInfo {
            user: user_id
        }).await??;

        assert_eq!(user.name, Some("Erhan".to_string()));

        /* Cleanup fields */
        user_manager.send(UpdateUser {
            user: user_id,
            name: Some("Erhan".to_string()),
            ..Default::default()
        }).await??;

        let user = user_manager.send(GetPublicUserInfo {
            user: user_id
        }).await??;

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
        let user_id = validate_auth(config, token.0).unwrap().user.id;

        let user = user_manager.send(GetDetailedUserInfo {
            user: user_id
        }).await??;
        
        assert_eq!(user.name, None);
        assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));
        
        user_manager.send(UpdateUser {
            user: user_id,
            name: Some("Erhan".to_string()),
            ..Default::default()
        }).await??;

        let user = user_manager.send(GetDetailedUserInfo {
            user: user_id
        }).await??;

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
        let user_id = validate_auth(config, token.0).unwrap().user.id;

        let user = user_manager.send(GetDetailedUserInfo {
            user: user_id
        }).await??;
        
        assert_eq!(user.name, None);
        assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));
        
        user_manager.send(UpdateUser {
            user: user_id,
            name: Some("Erhan".to_string()),
            ..Default::default()
        }).await??;

        let user = user_manager.send(GetDetailedUserInfo {
            user: user_id
        }).await??;

        assert_eq!(user.name, Some("Erhan".to_string()));
        assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));

        /* Cleanup fields */
        user_manager.send(UpdateUser {
            user: user_id,
            name: Some("Erhan".to_string()),
            ..Default::default()
        }).await??;

        let user = user_manager.send(GetDetailedUserInfo {
            user: user_id
        }).await??;

        assert_eq!(user.name, Some("Erhan".to_string()));
        assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));

        Ok(())
    }
}