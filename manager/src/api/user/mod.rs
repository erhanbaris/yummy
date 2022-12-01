pub mod model;

use std::time::Duration;
use std::{marker::PhantomData, ops::Deref};
use std::sync::Arc;
use database::model::{UserUpdate, PrivateUserModel};

use actix::{Context, Actor, Handler};
use database::{Pool, DatabaseTrait, RowId};

use general::meta::MetaType;
use moka::sync::Cache;
use uuid::Uuid;

use crate::response::Response;
use crate::api::auth::model::AuthError;

use self::model::*;

pub struct UserManager<DB: DatabaseTrait + ?Sized> {
    database: Arc<Pool>,
    _marker: PhantomData<DB>,

    // Caches
    cache_private_user_info: Cache<Uuid, PrivateUserModel>
}

impl<DB: DatabaseTrait + ?Sized> UserManager<DB> {
    pub fn new(database: Arc<Pool>) -> Self {
        Self {
            database,
            _marker: PhantomData,
            cache_private_user_info: Cache::builder().time_to_idle(Duration::from_secs(5*60)).build()
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Actor for UserManager<DB> {
    type Context = Context<Self>;
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static>  UserManager<DB> {
    fn cleanup_user_cache(&self, user_id: &Uuid) {
        self.cache_private_user_info.invalidate(user_id);
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<GetUserInformation> for UserManager<DB> {
    type Result = anyhow::Result<Response>;

    #[tracing::instrument(name="User::GetPrivateUser", skip(self, _ctx))]
    fn handle(&mut self, model: GetUserInformation, _ctx: &mut Context<Self>) -> Self::Result {

        let user_id = match model.requester_user.deref() {
            Some(user) => user.user.0,
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        };

        let user = DB::get_my_information(&mut self.database.get()?, user_id.into())?;

        match user {
            Some(user) => Ok(Response::UserPrivateInfo(user)),
            None => Err(anyhow::anyhow!(UserError::UserNotFound))
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

        if model.custom_id.is_none() && model.device_id.is_none() && model.email.is_none() && model.name.is_none() && model.password.is_none() && model.meta.as_ref().map(|dict| dict.len()).unwrap_or_default() == 0 {
            return Err(anyhow::anyhow!(UserError::UpdateInformationMissing));
        }

        let mut updates = UserUpdate::default();
        let user_id = RowId(original_user_id);

        let mut connection = self.database.get()?;
        let user = match DB::get_my_information(&mut connection, user_id)? {
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

        let mut connection = self.database.get()?;

        DB::transaction::<_, anyhow::Error, _>(&mut connection, |connection| {
            if let Some(meta) = model.meta {
                if meta.len() > 0 {
                    let user_old_metas = DB::get_user_meta(connection, user_id.into(), model.access_level)?;
                    let mut remove_list = Vec::new();
                    let mut insert_list = Vec::new();

                    for (key, value) in meta.into_iter() {
                        let row= user_old_metas.iter().find(|item| item.1 == key).map(|item| item.0);

                        /* Remove the key if exists in the database */
                        if let Some(row_id) = row {
                            remove_list.push(row_id);
                        }

                        /* Remove meta */
                        if let MetaType::Null = value {
                            continue;
                        }

                        insert_list.push((key, value));
                    }

                    DB::remove_user_metas(connection, remove_list)?;
                    DB::insert_user_metas(connection, user_id, insert_list)?;
                }
            }

            match DB::update_user(connection, user_id, updates)? {
                0 => Err(anyhow::anyhow!(UserError::UserNotFound)),
                _ => {
                    self.cleanup_user_cache(&original_user_id);
                    Ok(Response::None)
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use general::auth::UserAuth;
    use general::auth::validate_auth;
    use general::config::YummyConfig;
    use general::config::get_configuration;
    use general::meta::MetaAccess;
    use general::model::YummyState;
    use std::collections::HashMap;
    use std::env::temp_dir;
    use std::sync::Arc;

    use actix::Actor;
    use actix::Addr;
    use anyhow::Ok;
    use database::{create_database, create_connection};

    use super::*;
    use crate::api::auth::AuthManager;
    use crate::api::auth::model::*;

    fn create_actor() -> anyhow::Result<(Addr<UserManager<database::SqliteStore>>, Addr<AuthManager<database::SqliteStore>>, Arc<YummyConfig>)> {
        let mut db_location = temp_dir();
        db_location.push(format!("{}.db", Uuid::new_v4()));
        
        let config = get_configuration();
        let states = Arc::new(YummyState::default());
        let connection = create_connection(db_location.to_str().unwrap())?;
        create_database(&mut connection.clone().get()?)?;
        Ok((UserManager::<database::SqliteStore>::new(Arc::new(connection.clone())).start(), AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection)).start(), config))
    }
    
    #[actix::test]
    async fn get_private_user_1() -> anyhow::Result<()> {
        let (user_manager, _, _) = create_actor()?;
        let user = user_manager.send(GetUserInformation {
            requester_user: Arc::new(None),
            target_user: None
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
        let user = user_manager.send(GetUserInformation {
            requester_user: Arc::new(Some(UserAuth {
                user: user.user.id,
                session: user.user.session
            })),
            target_user: None
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

        let user = user_manager.send(GetUserInformation {
            requester_user: user_auth.clone(),
            target_user: None
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

        let user = user_manager.send(GetUserInformation {
            requester_user: user_auth.clone(),
            target_user: None
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

        let user = user_manager.send(GetUserInformation {
            requester_user: user_auth.clone(),
            target_user: None
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
            meta: Some(HashMap::from([
                ("gender".to_string(), MetaType::String("Male".to_string(), MetaAccess::Friend)),
                ("location".to_string(), MetaType::String("Copenhagen".to_string(), MetaAccess::Friend)),
                ("postcode".to_string(), MetaType::Integer(1000, MetaAccess::Mod)),
                ("score".to_string(), MetaType::Float(15.3, MetaAccess::Anonymous)),
                ("temp_admin".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ])),
            ..Default::default()
        }).await??;

        let user = user_manager.send(GetUserInformation {
            requester_user: user_auth.clone(),
            target_user: None
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

        let user = user_manager.send(GetUserInformation {
            requester_user: user_auth.clone(),
            target_user: None
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