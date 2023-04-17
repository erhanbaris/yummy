/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::{marker::PhantomData, sync::Arc, ops::Deref};

use yummy_cache::state::YummyState;
use yummy_database::DatabaseTrait;
use yummy_model::meta::collection::{UserMetaCollection, UserMetaCollectionInformation};
use yummy_model::request::RequestUserTypeVariant;
use yummy_model::user::UserUpdate;
use yummy_model::{UserId, UserType, UserInformationModel, UserMetaId};
use yummy_model::config::YummyConfig;
use yummy_model::meta::{UserMetaAccess, MetaType, UserMetaType};
use yummy_model::web::Answer;
use yummy_general::database::Pool;

use crate::{auth::model::AuthError, get_user_id_from_auth};

use super::model::{GetUserInformation, GetUserInformationEnum, UpdateUser};
use super::model::UserError;

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* **************************************************************************************************************** */
macro_rules! update_optional_property {
    ($updates: expr, $user_information: expr, $property: ident) => {
        $updates.$property = $property.as_ref().map(|item| {
            let result = match item.trim().is_empty() {
                true => None,
                false => Some(item.clone())
            };
            $user_information.$property = result.clone();
            result
        });
    }
}
/* **************************************************************************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
#[derive(Clone)]
pub struct UserLogic<DB: DatabaseTrait + ?Sized> {
    config: Arc<YummyConfig>,
    database: Arc<Pool>,
    states: YummyState,
    _marker: PhantomData<DB>
}

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* **************************************************************************************************************** */
impl<DB: DatabaseTrait + ?Sized> UserLogic<DB> {
    pub fn new(config: Arc<YummyConfig>, states: YummyState, database: Arc<Pool>) -> Self {
        Self {
            config,
            database,
            states,
            _marker: PhantomData
        }
    }

    fn get_user_access_level(&mut self, current_user_id: &UserId, target_user_id: &UserId) -> anyhow::Result<UserMetaAccess> {
        if current_user_id == target_user_id {
            return Ok(UserMetaAccess::Me);
        }

        match self.states.get_user_type(current_user_id)? {
            Some(UserType::User) => Ok(UserMetaAccess::User),
            Some(UserType::Mod) => Ok(UserMetaAccess::Mod),
            Some(UserType::Admin) => Ok(UserMetaAccess::Admin),
            None => Err(anyhow::anyhow!(UserError::UserNotFound))
        }
    }

    pub fn get_user_meta(&self, user_id: UserId, key: String) -> anyhow::Result<Option<UserMetaType>> {
        Ok(self.states.get_user_meta(&user_id, UserMetaAccess::System)?
            .get_with_name(&key)
            .cloned()
            .map(|item| item.meta))
    }

    pub fn get_user_metas(&self, user_id: UserId) -> anyhow::Result<Vec<UserMetaCollectionInformation>> {
        Ok(self.states.get_user_metas(&user_id)?)
    }

    pub fn set_user_meta(&self, user_id: UserId, key: String, value: UserMetaType) -> anyhow::Result<()> {
        Ok(self.states.set_user_meta(&user_id, key, value)?)
    }

    pub fn remove_all_metas(&self, user_id: UserId) -> anyhow::Result<()> {
        Ok(self.states.remove_all_user_metas(&user_id)?)
    }

    pub fn remove_user_meta(&self, user_id: UserId, key: String) -> anyhow::Result<()> {
        Ok(self.states.remove_user_meta(&user_id, key)?)
    }

    pub fn get_user_information(&mut self, model: &GetUserInformation) -> anyhow::Result<UserInformationModel> {
        #[allow(unused_mut)]
        let mut execute = |user_id: &UserId, access_type: UserMetaAccess| -> anyhow::Result<UserInformationModel> {
            
            let user = self.states.get_user_information(user_id, access_type)?;

            match user {
                Some(mut user) => {
                    user.online = self.states.is_user_online(user_id);
                    Ok(user)
                },
                None => Err(anyhow::anyhow!(UserError::UserNotFound))
            }
        };

        match &model.query {
            GetUserInformationEnum::Me(user) => match user.deref() {
                Some(user) => execute(&user.user, UserMetaAccess::Me),
                None => Err(anyhow::anyhow!(AuthError::TokenNotValid))
            },
            GetUserInformationEnum::UserViaSystem(user) => execute(user, UserMetaAccess::System),
            GetUserInformationEnum::User { user, requester } => {
                match requester.deref() {
                    Some(requester) => {
                        let user_type = self.states.get_user_type(&requester.user)?;
                        execute(user, match user_type.unwrap_or_default() {
                            UserType::Admin => UserMetaAccess::Admin,
                            UserType::Mod => UserMetaAccess::Mod,
                            UserType::User => UserMetaAccess::User
                        })
                    },
                    None => execute(user, UserMetaAccess::Anonymous)
                }
            }
        }
    }

    pub fn update_user(&mut self, model: &UpdateUser) -> anyhow::Result<Answer> {
        let UpdateUser { name, email, password, device_id, custom_id, user_type, metas, meta_action, target_user_id, .. } = &model;

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

        let mut user_information = match self.states.get_user_information(target_user_id, user_access_level.clone())? {
            Some(user) => user,
            None => return Err(anyhow::anyhow!(UserError::UserNotFound))
        };

        update_optional_property!(updates, user_information, custom_id);
        update_optional_property!(updates, user_information, device_id);
        update_optional_property!(updates, user_information, name);

        updates.user_type = user_type.map(|item| {
            user_information.user_type = item;
            item.into()
        });

        if let Some(password) = password {
            if password.trim().len() < 4 {
                return Err(anyhow::anyhow!(UserError::PasswordIsTooSmall))
            }
            updates.password = Some(password.clone());
        }

        if let Some(email) = email {
            if user_information.email.is_some() {
                return Err(anyhow::anyhow!(UserError::CannotChangeEmail));
            }

            user_information.email = Some(email.clone());
            updates.email = Some(email.clone());
        }

        let config = self.config.clone();

        DB::transaction::<_, anyhow::Error, _>(&mut connection, |connection| {

            let meta_action = meta_action.clone();
            let user_access_level_code = user_access_level.clone() as u8;

            let (to_be_inserted, to_be_removed, total_metas) = match meta_action {

                // Dont remove old metas
                yummy_model::meta::MetaAction::OnlyAddOrUpdate => {

                    // Check for metas
                    match metas {
                        Some(metas) => {

                            let user_old_metas = self.states.get_user_meta(target_user_id, user_access_level.clone())?;
                            let mut remove_list = Vec::new();
                            let mut insert_list = Vec::new();

                            for (key, value) in metas {

                                let meta_access_level = value.get_access_level() as u8;
                                if meta_access_level > user_access_level_code {
                                    return Err(anyhow::anyhow!(UserError::MetaAccessLevelCannotBeBiggerThanUsersAccessLevel(key.clone())));
                                }

                                // Check for meta already added into the user
                                let row = user_old_metas.iter().find(|item| &item.name == key).map(|item| (item.name.clone(), item.id.clone()));
        
                                /* Remove the key if exists in the database */
                                if let Some(row_info) = row {
                                    remove_list.push(row_info);
                                }
        
                                /* 
                                If the meta value is Null, skip to add insert_list.
                                We already add all metas into the remove_list variable.
                                */
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
                yummy_model::meta::MetaAction::RemoveUnusedMetas => {

                    // Check for metas
                    match metas {
                        Some(metas) => {
                            let remove_list = self.states.get_user_meta(target_user_id, user_access_level.clone())?.into_iter().map(|meta| (meta.name, meta.id)).collect::<Vec<_>>();
                            let mut insert_list = Vec::new();

                            for (key, value) in metas {
                                
                                let meta_access_level = value.get_access_level() as u8;
                                if meta_access_level > user_access_level_code {
                                    return Err(anyhow::anyhow!(UserError::MetaAccessLevelCannotBeBiggerThanUsersAccessLevel(key.clone())));
                                }

                                /* 
                                If the meta value is Null, skip to add insert_list.
                                We already add all metas into the remove_list variable.
                                */
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
                yummy_model::meta::MetaAction::RemoveAllMetas => {
                    // Discard all new meta insertion list and remove all old meta that based on user access level.
                    (None, Some(self.states.get_user_meta(target_user_id, user_access_level)?.into_iter().map(|meta| (meta.name, meta.id)).collect::<Vec<_>>()), 0)
                },    
            };

            if total_metas > config.max_user_meta {
                return Err(anyhow::anyhow!(UserError::MetaLimitOverToMaximum));
            }

            /* Remove metas from database and cache */
            if let Some(to_be_removed) = to_be_removed {
                DB::remove_user_metas(connection, to_be_removed.iter().filter_map(|item| item.1.clone()).collect::<Vec<UserMetaId>>())?;

                // If the metas are already None, no need to do anything
                if let Some(metas) = user_information.metas.as_mut() {
                    for (name, _) in to_be_removed.into_iter() {

                        // Find to be removed meta in the users meta list 
                        metas.remove_with_name(&name);
                    }
                }

                /* Set metas to None if the there is no records at metas */
                let is_empty = user_information.metas.as_ref().map_or(false, |item| item.is_empty());
                if is_empty {
                    user_information.metas = None;
                }
            }

            if let Some(to_be_inserted) = to_be_inserted {
                DB::insert_user_metas(connection, target_user_id, to_be_inserted.clone())?;

                user_information.metas = match user_information.metas {

                    // Add new metas to current cache
                    Some(mut metas) => {
                        for (name, value) in to_be_inserted.into_iter() {
                            metas.add(name.clone(), value.clone());
                        }
                        Some(metas)
                    }

                    // Metas cache is None, create new meta hashmap and new metas to current cache
                    None => {
                        let mut metas = UserMetaCollection::new();
                        for (name, value) in to_be_inserted.into_iter() {
                            metas.add(name.clone(), value.clone());
                        }
                        Some(metas)
                    }
                };
            }
            
            let response = match has_user_update {
                true => match DB::update_user(connection, target_user_id, &updates)? {
                    0 => return Err(anyhow::anyhow!(UserError::UserNotFound)),
                    _ => Answer::success(model.request_id, RequestUserTypeVariant::Update)
                },
                false => Answer::success(model.request_id, RequestUserTypeVariant::Update)
            };

            // Update user cache
            self.states.update_user_information(target_user_id, user_information)?;

            Ok(response)
        })
    }
}

/* **************************************************************************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */
