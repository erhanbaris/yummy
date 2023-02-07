pub mod model;

#[cfg(test)]
mod test;

use std::{marker::PhantomData, ops::Deref};
use std::sync::Arc;
use database::model::UserUpdate;

use actix::{Context, Actor, Handler};
use database::{Pool, DatabaseTrait};

use general::config::YummyConfig;
use general::meta::{MetaType, UserMetaAccess};
use general::model::{UserType, UserId};
use general::state::YummyState;
use general::web::{GenericAnswer, Answer};

use crate::auth::model::AuthError;
use crate::get_user_id_from_auth;
use crate::plugin::PluginExecuter;

use self::model::*;

pub struct UserManager<DB: DatabaseTrait + ?Sized> {
    config: Arc<YummyConfig>,
    database: Arc<Pool>,
    states: YummyState,
    executer: Arc<PluginExecuter>,
    _marker: PhantomData<DB>
}

impl<DB: DatabaseTrait + ?Sized> UserManager<DB> {
    pub fn new(config: Arc<YummyConfig>, states: YummyState, database: Arc<Pool>, executer: Arc<PluginExecuter>) -> Self {
        Self {
            config,
            database,
            states,
            executer,
            _marker: PhantomData
        }
    }

    fn get_user_access_level(&mut self, current_user_id: &UserId, target_user_id: &UserId) -> anyhow::Result<UserMetaAccess> {
        if current_user_id == target_user_id {
            return Ok(UserMetaAccess::Me);
        }

        match self.states.get_user_type(current_user_id) {
            Some(UserType::User) => Ok(UserMetaAccess::User),
            Some(UserType::Mod) => Ok(UserMetaAccess::Mod),
            Some(UserType::Admin) => Ok(UserMetaAccess::Admin),
            None => Err(anyhow::anyhow!(UserError::UserNotFound))
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Actor for UserManager<DB> {
    type Context = Context<Self>;
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<GetUserInformation> for UserManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="GetUserInformation", skip(self, _ctx))]
    #[macros::plugin_api(name="get_user_information")]
    fn handle(&mut self, model: GetUserInformation, _ctx: &mut Context<Self>) -> Self::Result {

        let mut connection = self.database.get()?;

        #[allow(unused_mut)]
        let mut execute = |connection, user_id, access_type| -> anyhow::Result<()> {
            let user = DB::get_user_information(connection, user_id, access_type)?;
            match user {
                Some(mut user) => {
                    user.online = self.states.is_user_online(user_id);
                    model.socket.send(GenericAnswer::success(UserResponse::UserInfo { user }).into());
                    Ok(())
                },
                None => Err(anyhow::anyhow!(UserError::UserNotFound))
            }
        };

        match &model.query {
            GetUserInformationEnum::Me(user) => match user.deref() {
                Some(user) => execute(&mut connection, &user.user, UserMetaAccess::Me),
                None => Err(anyhow::anyhow!(AuthError::TokenNotValid))
            },
            GetUserInformationEnum::UserViaSystem(user) => execute(&mut connection, user, UserMetaAccess::System),
            GetUserInformationEnum::User { user, requester } => {
                match requester.deref() {
                    Some(requester) => {
                        let user_type = DB::get_user_type(&mut connection, &requester.user)?;
                        execute(&mut connection, user, match user_type {
                            UserType::Admin => UserMetaAccess::Admin,
                            UserType::Mod => UserMetaAccess::Mod,
                            UserType::User => UserMetaAccess::User
                        })
                    },
                    None => execute(&mut connection, user, UserMetaAccess::Anonymous)
                }
            }
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<UpdateUser> for UserManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="UpdateUser", skip(self, _ctx))]
    #[macros::plugin_api(name="update_user")]
    fn handle(&mut self, model: UpdateUser, _ctx: &mut Context<Self>) -> Self::Result {
        let UpdateUser { name, socket, email, password, device_id, custom_id, user_type, metas, meta_action, target_user_id, .. } = &model;

        let user_id = get_user_id_from_auth!(model);

        let target_user_id = match &target_user_id {
            Some(target_user_id) => target_user_id,
            None => user_id
        };

        let has_user_update = custom_id.is_some() || device_id.is_some() || email.is_some() || name.is_some() || password.is_some() || user_type.is_some();

        if !has_user_update && metas.as_ref().map(|dict| dict.len()).unwrap_or_default() == 0 {
            return Err(anyhow::anyhow!(UserError::UpdateInformationMissing));
        }

        let mut updates = UserUpdate::default();

        let mut connection = self.database.get()?;
        let user_access_level = self.get_user_access_level(user_id, target_user_id)?;

        let user = match DB::get_user_information(&mut connection, target_user_id, user_access_level.clone())? {
            Some(user) => user,
            None => return Err(anyhow::anyhow!(UserError::UserNotFound))
        };

        // Todo: dont use clone
        updates.custom_id = custom_id.as_ref().map(|item| match item.trim().is_empty() { true => None, false => Some(item.clone())} );
        updates.device_id = device_id.as_ref().map(|item| match item.trim().is_empty() { true => None, false => Some(item.clone())} );
        updates.name = name.as_ref().map(|item| match item.trim().is_empty() { true => None, false => Some(item.clone())} );
        updates.user_type = user_type.map(|item| item.into());

        if let Some(password) = password {
            if password.trim().len() < 4 {
                return Err(anyhow::anyhow!(UserError::PasswordIsTooSmall))
            }
            updates.password = Some(password.clone());
        }

        if let Some(email) = email {
            if user.email.is_some() {
                return Err(anyhow::anyhow!(UserError::CannotChangeEmail));
            }
            updates.email = Some(email.clone())
        }

        let config = self.config.clone();

        DB::transaction::<_, anyhow::Error, _>(&mut connection, |connection| {

            let meta_action = meta_action.clone().unwrap_or_default();
            let user_access_level_code = user_access_level.clone() as u8;

            let (to_be_inserted, to_be_removed, total_metas) = match meta_action {

                // Dont remove old metas
                general::meta::MetaAction::OnlyAddOrUpdate => {

                    // Check for metas
                    match metas {
                        Some(metas) => {
                            let user_old_metas = DB::get_user_meta(connection, target_user_id, user_access_level)?;
                            let mut remove_list = Vec::new();
                            let mut insert_list = Vec::new();

                            for (key, value) in metas {

                                let meta_access_level = value.get_access_level() as u8;
                                if meta_access_level > user_access_level_code {
                                    return Err(anyhow::anyhow!(UserError::MetaAccessLevelCannotBeBiggerThanUsersAccessLevel(key.clone())));
                                }

                                // Check for meta already added into the user
                                let row = user_old_metas.iter().find(|item| &item.1 == key).map(|item| item.0.clone());
        
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
                            
                            let total_metas = (user_old_metas.len() - remove_list.len()) + insert_list.len();
                            let insert_list = (!insert_list.is_empty()).then_some(insert_list);
                            let remove_list = (!remove_list.is_empty()).then_some(remove_list);

                            (insert_list, remove_list, total_metas)
                        },
                        None => (None, None, 0)
                    }
                },

                // Add ne metas than remove all old meta informations
                general::meta::MetaAction::RemoveUnusedMetas => {

                    // Check for metas
                    match metas {
                        Some(metas) => {
                            let remove_list = DB::get_user_meta(connection, target_user_id, user_access_level.clone())?.into_iter().map(|meta| meta.0).collect::<Vec<_>>();
                            let mut insert_list = Vec::new();

                            for (key, value) in metas {
                                
                                let meta_access_level = value.get_access_level() as u8;
                                if meta_access_level > user_access_level_code {
                                    return Err(anyhow::anyhow!(UserError::MetaAccessLevelCannotBeBiggerThanUsersAccessLevel(key.clone())));
                                }

                                if let MetaType::Null = value {
                                    continue;
                                }
        
                                insert_list.push((key, value));
                            }
                            
                            let total_metas = insert_list.len();
                            let insert_list = (!insert_list.is_empty()).then_some(insert_list);
                            let remove_list = (!remove_list.is_empty()).then_some(remove_list);

                            (insert_list, remove_list, total_metas)
                        },
                        None => (None, None, 0)
                    }
                },
                general::meta::MetaAction::RemoveAllMetas => {
                    // Discard all new meta insertion list and remove all old meta that based on user access level.
                    (None, Some(DB::get_user_meta(connection, target_user_id, user_access_level)?.into_iter().map(|meta| meta.0).collect::<Vec<_>>()), 0)
                },    
            };

            if total_metas > config.max_user_meta {
                return Err(anyhow::anyhow!(UserError::MetaLimitOverToMaximum));
            }

            if let Some(to_be_removed) = to_be_removed {
                DB::remove_user_metas(connection, to_be_removed)?;
            }

            if let Some(to_be_inserted) = to_be_inserted {
                DB::insert_user_metas(connection, target_user_id, to_be_inserted)?;
            }
            
            // Update user
            match has_user_update {
                true => match DB::update_user(connection, target_user_id, &updates)? {
                    0 => return Err(anyhow::anyhow!(UserError::UserNotFound)),
                    _ => socket.send(Answer::success().into())
                },
                false => socket.send(Answer::success().into())
            };

            // todo: convert to single execution
            if let Some(user_type) = user_type {
                self.states.set_user_type(target_user_id, *user_type);
            }

            if let Some(Some(name)) = updates.name {
                self.states.set_user_name(target_user_id, name);
            }
            Ok(())
        })
    }
}
