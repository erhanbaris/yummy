pub mod model;

#[cfg(test)]
mod test;

use std::time::Duration;
use std::{marker::PhantomData, ops::Deref};
use std::sync::Arc;
use database::model::{UserUpdate, UserInformationModel};

use actix::{Context, Actor, Handler};
use database::{Pool, DatabaseTrait, RowId};

use general::config::YummyConfig;
use general::meta::{MetaType, MetaAccess};
use general::model::{UserType, YummyState, UserId};
use moka::sync::Cache;
use uuid::Uuid;

use crate::response::Response;
use crate::api::auth::model::AuthError;

use self::model::*;

pub struct UserManager<DB: DatabaseTrait + ?Sized> {
    config: Arc<YummyConfig>,
    database: Arc<Pool>,
    states: Arc<YummyState>,
    _marker: PhantomData<DB>,

    // Caches
    cache_private_user_info: Cache<Uuid, UserInformationModel>
}

impl<DB: DatabaseTrait + ?Sized> UserManager<DB> {
    pub fn new(config: Arc<YummyConfig>, states: Arc<YummyState>, database: Arc<Pool>) -> Self {
        Self {
            config,
            database,
            states,
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

    #[tracing::instrument(name="User::get user info", skip(self, _ctx))]
    fn handle(&mut self, model: GetUserInformation, _ctx: &mut Context<Self>) -> Self::Result {

        let mut connection = self.database.get()?;

        let (user_id, access_type) = match model.query {
            GetUserInformationEnum::Me(user) => match user.deref() {
                Some(user) => (user.user.get(), MetaAccess::Me),
                None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
            },
            GetUserInformationEnum::UserViaSystem(user) => (user.get(), MetaAccess::System),
            GetUserInformationEnum::User { user, requester } => {
                match requester.deref() {
                    Some(requester) => {
                        let user_type = DB::get_user_type(&mut connection, RowId(requester.user.get()))?;
                        (user.get(), match user_type {
                            UserType::Admin => MetaAccess::Admin,
                            UserType::Mod => MetaAccess::Mod,
                            UserType::User => MetaAccess::User
                        })
                    },
                    None => (user.get(), MetaAccess::Anonymous)
                }
            }
        };

        let user = DB::get_user_information(&mut connection, user_id.into(), access_type)?;
        match user {
            Some(mut user) => {
                user.online = self.states.is_user_online(UserId::from(user_id));
                Ok(Response::UserInformation(user))
            },
            None => Err(anyhow::anyhow!(UserError::UserNotFound))
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<UpdateUser> for UserManager<DB> {
    type Result = anyhow::Result<Response>;

    #[tracing::instrument(name="User::UpdateUser", skip(self, _ctx))]
    fn handle(&mut self, model: UpdateUser, _ctx: &mut Context<Self>) -> Self::Result {

        let original_user_id = match model.user.deref() {
            Some(user) => user.user.get(),
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        };

        let user_updated = model.custom_id.is_some() || model.device_id.is_some() || model.email.is_some() || model.name.is_some() || model.password.is_some() || model.user_type.is_some();

        if !user_updated && model.meta.as_ref().map(|dict| dict.len()).unwrap_or_default() == 0 {
            return Err(anyhow::anyhow!(UserError::UpdateInformationMissing));
        }

        let mut updates = UserUpdate::default();
        let user_id = RowId(original_user_id);

        let mut connection = self.database.get()?;
        let user = match DB::get_user_information(&mut connection, user_id, MetaAccess::Admin)? {
            Some(user) => user,
            None => return Err(anyhow::anyhow!(UserError::UserNotFound))
        };

        updates.custom_id = model.custom_id.map(|item| match item.trim().is_empty() { true => None, false => Some(item)} );
        updates.device_id = model.device_id.map(|item| match item.trim().is_empty() { true => None, false => Some(item)} );
        updates.name = model.name.map(|item| match item.trim().is_empty() { true => None, false => Some(item)} );
        updates.user_type = model.user_type.map(|item| item.into());

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
        let config = self.config.clone();

        DB::transaction::<_, anyhow::Error, _>(&mut connection, move |connection| {
            if let Some(meta) = model.meta {

                match meta.len() {
                    0 => (),
                    n if n > config.max_user_meta => return Err(anyhow::anyhow!(UserError::MetaLimitOverToMaximum)),
                    _ => {
                        let user_old_metas = DB::get_user_meta(connection, user_id, model.access_level)?;
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
                };
            }

            // Update user
            match user_updated {
                true => match DB::update_user(connection, user_id, updates)? {
                    0 => Err(anyhow::anyhow!(UserError::UserNotFound)),
                    _ => {
                        self.cleanup_user_cache(&original_user_id);
                        Ok(Response::None)
                    }
                },
                false => Ok(Response::None)
            }
        })
    }
}
