/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* **************************************************************************************************************** */
pub mod model;

/* **************************************************************************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
#[cfg(test)]
mod test;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{marker::PhantomData, ops::Deref};
use std::sync::Arc;
use actix::{Context, Actor, Handler};
use actix_broker::{BrokerSubscribe, BrokerIssue};
use anyhow::anyhow;
use yummy_cache::state::{RoomInfoTypeVariant, YummyState, RoomInfoType};
use yummy_database::DatabaseTrait;

use yummy_model::config::YummyConfig;
use yummy_model::meta::{MetaType, MetaAction};
use yummy_model::meta::RoomMetaAccess;
use yummy_model::user::RoomUpdate;
use yummy_model::{RoomId, UserId, RoomUserType, UserType, SessionId, SendMessage};
use yummy_model::web::{GenericAnswer, Answer};
use yummy_general::database::Pool;
use yummy_general::database::PooledConnection;

use crate::auth::model::{AuthError, RoomUserDisconnect};
use crate::plugin::PluginExecuter;
use crate::{get_user_session_id_from_auth, get_user_id_from_auth, get_session_id_from_auth};
use crate::user::model::UserError;

use self::model::*;

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************************************************************************** */
const ALL_ROOM_INFO_TYPE_VARIANTS: [RoomInfoTypeVariant; 10] = [RoomInfoTypeVariant::Tags, RoomInfoTypeVariant::InsertDate, RoomInfoTypeVariant::RoomName, RoomInfoTypeVariant::AccessType, RoomInfoTypeVariant::Users, RoomInfoTypeVariant::MaxUser, RoomInfoTypeVariant::UserLength, RoomInfoTypeVariant::BannedUsers, RoomInfoTypeVariant::JoinRequest, RoomInfoTypeVariant::Metas];

type ConfigureMetasResult = anyhow::Result<(Option<HashMap<String, MetaType<RoomMetaAccess>>>, HashMap<String, MetaType<RoomMetaAccess>>)>;


/* **************************************************************************************************************** */
/* **************************************************** MACROS **************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
pub struct RoomManager<DB: DatabaseTrait + ?Sized> {
    config: Arc<YummyConfig>,
    database: Arc<Pool>,
    states: YummyState,
    executer: Arc<PluginExecuter>,
    _marker: PhantomData<DB>
}

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* **************************************************************************************************************** */
impl<DB: DatabaseTrait + ?Sized> RoomManager<DB> {
    pub fn new(config: Arc<YummyConfig>, states: YummyState, database: Arc<Pool>, executer: Arc<PluginExecuter>) -> Self {
        Self {
            config,
            database,
            states,
            executer,
            _marker: PhantomData
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> RoomManager<DB> {
    fn join_to_room(&mut self, connection: &mut PooledConnection, request_id: Option<usize>, room_id: &RoomId, user_id: &UserId, session_id: &SessionId, room_user_type: RoomUserType) -> anyhow::Result<()> {
        /* Room does not require approvement */
        let users = self.states.get_users_from_room(room_id)?;
        self.states.join_to_room(room_id, user_id, session_id, room_user_type.clone())?;
        
        DB::join_to_room(connection, room_id, user_id, room_user_type)?;

        let message = serde_json::to_string(&RoomResponse::UserJoinedToRoom {
            user: user_id,
            room: room_id
        }).unwrap();

        for user_id in users.into_iter() {
            self.issue_system_async(SendMessage {
                message: message.clone(),
                user_id
            });
        }

        let access_level = self.get_access_level_for_room(user_id, session_id, room_id)?;
        
        let infos = self.states.get_room_info(room_id, access_level, &[RoomInfoTypeVariant::RoomName, RoomInfoTypeVariant::Users, RoomInfoTypeVariant::Metas])?;
        let room_name = infos.get_room_name();
        let users = infos.get_users();
        let metas = infos.get_metas();
        
        self.issue_system_async(SendMessage {
            user_id: Arc::new(user_id.clone()),
            message: GenericAnswer::success(request_id, RoomResponse::Joined { room_name, users, metas, room: room_id }).into()
        });
        Ok(())
    }

    fn disconnect_from_room(&mut self, room_id: &RoomId, user_id: &UserId, session_id: &SessionId) -> anyhow::Result<bool> {
        let room_removed = self.states.disconnect_from_room(room_id, user_id, session_id)?;
        let users = self.states.get_users_from_room(room_id)?;
        
        let mut connection = self.database.get()?;
        DB::disconnect_from_room(&mut connection, room_id, user_id)?;

        let message = serde_json::to_string(&RoomResponse::UserDisconnectedFromRoom {
            user: user_id,
            room: room_id
        }).unwrap();

        for user_id in users.into_iter() {
            self.issue_system_async(SendMessage {
                message: message.clone(),
                user_id
            });
        }

        Ok(room_removed)
    }

    fn configure_metas(&self, connection: &mut PooledConnection, room_id: &RoomId, metas: Option<HashMap<String, MetaType<RoomMetaAccess>>>, meta_action: MetaAction, access_level: RoomMetaAccess) -> ConfigureMetasResult {
        let room_access_level_code = access_level as u8;
        let mut metas = metas;

        let (to_be_inserted_metas, to_be_removed_metas, total_metas, remaining) = match meta_action {

            // Dont remove old metas
            yummy_model::meta::MetaAction::OnlyAddOrUpdate => {

                // Check for metas
                if let Some(ref metas) = metas {
                    let mut room_old_metas = DB::get_room_meta(connection, room_id, access_level)?;
                    let mut remove_list = Vec::new();
                    let mut insert_list = Vec::new();

                    for (key, value) in metas.iter() {

                        let meta_access_level = value.get_access_level() as u8;
                        if meta_access_level > room_access_level_code {
                            return Err(anyhow::anyhow!(UserError::MetaAccessLevelCannotBeBiggerThanUsersAccessLevel(key.clone())));
                        }

                        // Check for meta already added into the user
                        let row = room_old_metas.iter().enumerate().find(|(_, item)| &item.1 == key).map(|(index, (item, _, _))| (index, item.clone()));

                        /* Remove the key if exists in the database */
                        if let Some((index, row_id)) = row {
                            remove_list.push(row_id);
                            room_old_metas.remove(index);
                        }

                        /* Remove meta */
                        if let MetaType::Null = value {
                            continue;
                        }

                        insert_list.push((key, value));
                    }
                    
                    let total_metas = (room_old_metas.len().checked_sub(remove_list.len()).unwrap_or_default()) + insert_list.len();
                    let insert_list = (!insert_list.is_empty()).then_some(insert_list);
                    let remove_list = (!remove_list.is_empty()).then_some(remove_list);

                    (insert_list, remove_list, total_metas, room_old_metas.into_iter().map(|(_, key, value)| (key, value)).collect::<HashMap<_, _>>())
                } else {
                    (None, None, 0, HashMap::default())
                }
            },

            // Add new metas than remove all old meta informations
            yummy_model::meta::MetaAction::RemoveUnusedMetas => {

                // Check for metas
                if let Some(ref metas) = metas {
                    let remove_list = DB::get_room_meta(connection, room_id, access_level)?.into_iter().map(|meta| meta.0).collect::<Vec<_>>();
                    let mut insert_list = Vec::new();

                    for (key, value) in metas.iter() {
                        
                        let meta_access_level = value.get_access_level() as u8;
                        if meta_access_level > room_access_level_code {
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

                    (insert_list, remove_list, total_metas, HashMap::default())
                } else {
                    (None, None, 0, HashMap::default())
                }
            },
            yummy_model::meta::MetaAction::RemoveAllMetas => {
                // Discard all new meta insertion list and remove all old meta that based on user access level.
                let remove_list = DB::get_room_meta(connection, room_id, access_level)?.into_iter().map(|meta| meta.0).collect::<Vec<_>>();
                metas = Some(HashMap::default());
                (None, Some(remove_list), 0, HashMap::default())
            },
        };

        if total_metas > self.config.max_user_meta {
            return Err(anyhow::anyhow!(UserError::MetaLimitOverToMaximum));
        }

        if let Some(to_be_removed_metas) = to_be_removed_metas {
            DB::remove_room_metas(connection, to_be_removed_metas)?;
        }

        if let Some(to_be_inserted_metas) = to_be_inserted_metas {
            DB::insert_room_metas(connection, room_id, &to_be_inserted_metas)?;
        }

        Ok((metas, remaining))
    }

    fn configure_tags(&self, connection: &mut PooledConnection, room_id: &RoomId, tags: &Option<Vec<String>>) -> anyhow::Result<()> {
        if let Some(to_be_inserted_tags) = tags {
            let to_be_removed_tags = DB::get_room_tag(connection, room_id)?.into_iter().map(|item| item.0).collect::<Vec<_>>();
            
            if !to_be_removed_tags.is_empty() {
                DB::remove_room_tags(connection, to_be_removed_tags)?;
            }
            
            if !to_be_inserted_tags.is_empty() {
                DB::insert_room_tags(connection, room_id, to_be_inserted_tags)?;
            }    
        }

        Ok(())
    }

    fn get_access_level_for_room(&mut self, user_id: &UserId, session_id: &SessionId, room_id: &RoomId) -> anyhow::Result<RoomMetaAccess> {
        match self.states.get_user_type(user_id)? {
            Some(UserType::User) => match self.states.get_users_room_type(session_id, room_id)? {
                Some(RoomUserType::User) => Ok(RoomMetaAccess::User),
                Some(RoomUserType::Moderator) => Ok(RoomMetaAccess::Moderator),
                Some(RoomUserType::Owner) => Ok(RoomMetaAccess::Owner),
                None => Err(anyhow::anyhow!(UserError::UserNotBelongToRoom))
            },
            Some(UserType::Mod) => Ok(RoomMetaAccess::Moderator),
            Some(UserType::Admin) => Ok(RoomMetaAccess::Admin),
            None => Err(anyhow::anyhow!(UserError::UserNotFound))
        }
    }
}

/* **************************************************************************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* **************************************************************************************************************** */
impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Actor for RoomManager<DB> {
    type Context = Context<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context) {
        self.subscribe_system_async::<RoomUserDisconnect>(ctx);
        self.subscribe_system_async::<DisconnectFromRoomRequest>(ctx);
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<RoomUserDisconnect> for RoomManager<DB> {
    type Result = ();

    #[tracing::instrument(name="Room::User RoomUserDisconnect", skip(self, _ctx))]
    fn handle(&mut self, model: RoomUserDisconnect, _ctx: &mut Self::Context) -> Self::Result {

        if let Some(user) = model.auth.deref() {
            let rooms = self.states.get_user_rooms(&user.session);
    
            if let Some(rooms) = rooms {
                for room in rooms.into_iter() {
                    self.disconnect_from_room(&room, &user.user, &user.session).unwrap_or_default();
                }
            }
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<CreateRoomRequest> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="CreateRoom", skip(self, _ctx))]
    #[yummy_macros::plugin_api(name="create_room")]
    fn handle(&mut self, model: CreateRoomRequest, _ctx: &mut Context<Self>) -> Self::Result {
        
        // Check user information
        let (user_id, session_id) = get_user_session_id_from_auth!(model);

        let mut connection = self.database.get()?;

        let room_id = DB::transaction(&mut connection, |connection| {
            let insert_date = SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default();
            let room_id = DB::create_room(connection, model.name.clone(), model.access_type.clone(), model.max_user, model.join_request, &model.tags)?;

            DB::join_to_room(connection, &room_id, user_id, RoomUserType::Owner)?;

            let access_level = match self.states.get_user_type(user_id)? {
                Some(UserType::User) => RoomMetaAccess::Owner,
                Some(UserType::Mod) => RoomMetaAccess::Owner,
                Some(UserType::Admin) => RoomMetaAccess::Admin,
                None => return Err(anyhow::anyhow!(UserError::UserNotFound))
            };
            
            #[allow(unused_mut)]
            let (mut meta, _) = self.configure_metas(connection, &room_id, model.metas.clone(), MetaAction::OnlyAddOrUpdate, access_level)?;
            
            self.states.create_room(&room_id, insert_date, model.name.clone(), model.description.clone(), model.access_type.clone(), model.max_user, model.tags.clone(), meta, model.join_request);
            self.states.join_to_room(&room_id, user_id, session_id, RoomUserType::Owner)?;
           
            anyhow::Ok(room_id)
        })?;
        

        model.socket.send(GenericAnswer::success(model.request_id, RoomResponse::RoomCreated { room: room_id }).into());
        Ok(())
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<UpdateRoom> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="UpdateRoom", skip(self, _ctx))]
    #[yummy_macros::plugin_api(name="update_room")]
    fn handle(&mut self, model: UpdateRoom, _ctx: &mut Context<Self>) -> Self::Result {

        let (user_id, session_id) = get_user_session_id_from_auth!(model);

        let has_room_update = model.access_type.is_some() || model.max_user.is_some() || model.name.is_some() || model.description.is_some();

        if !has_room_update && model.metas.is_none() {
            return Err(anyhow::anyhow!(RoomError::UpdateInformationMissing));
        }

        // Calculate room access level for user
        let access_level = self.get_access_level_for_room(user_id, session_id, &model.room_id)?;

        let mut connection = self.database.get()?;

        DB::transaction::<_, anyhow::Error, _>(&mut connection, |connection| {

            /* Meta configuration */
            #[allow(unused_mut)]
            let (mut metas, mut remaining) = self.configure_metas(connection, &model.room_id, model.metas.clone(), model.meta_action.clone(), access_level)?;

            /* Tag configuration */
            let tags = model.tags.clone();
            self.configure_tags(connection, &model.room_id, &tags)?;

            /* Change user permission */
            if let Some(user_permission) = &model.user_permission {
                DB::update_room_user_permissions(connection, &model.room_id, user_permission)?;
                
                for (user_id, user_type) in user_permission {
                    self.states.set_users_room_type(user_id, &model.room_id, user_type.clone())?;
                }
            }
            
            // Update user
            let updates = RoomUpdate {
                max_user: model.max_user.map(|item| item as i32 ),
                access_type: model.access_type.as_ref().map(|item| item.clone().into() ),
                join_request: model.join_request.map(|item| item.into() ),
                name: model.name.as_ref().map(|item| match item.trim().is_empty() { true => None, false => Some(&item[..])} ),
                description: model.description.as_ref().map(|item| match item.trim().is_empty() { true => None, false => Some(&item[..])} )
            };

            match has_room_update {
                true => match DB::update_room(connection, &model.room_id, &updates)? {
                    0 => return Err(anyhow::anyhow!(UserError::UserNotFound)),
                    _ => model.socket.send(Answer::success(model.request_id).into())
                },
                false => model.socket.send(Answer::success(model.request_id).into())
            };

            // Update all caches
            let mut room_update_query = Vec::new();
            if let Some(name) = updates.name {
                room_update_query.push(RoomInfoType::RoomName(name.map(|item| item.to_string())));
            }
            if let Some(description) = updates.description {
                room_update_query.push(RoomInfoType::Description(description.map(|item| item.to_string())));
            }

            if let Some(max_user) = updates.max_user {
                room_update_query.push(RoomInfoType::MaxUser(max_user as usize));
            }

            if let Some(tags) = tags {
                room_update_query.push(RoomInfoType::Tags(tags));
            }

            if let Some(mut metas) = metas {
                metas.extend(remaining.into_iter());
                room_update_query.push(RoomInfoType::Metas(metas));
            }

            if !room_update_query.is_empty() {
                self.states.set_room_info(&model.room_id, room_update_query);
            }
            Ok(())
        })
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<WaitingRoomJoins> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="WaitingRoomJoins", skip(self, _ctx))]
    #[yummy_macros::plugin_api(name="waiting_room_joins")]
    fn handle(&mut self, model: WaitingRoomJoins, _ctx: &mut Context<Self>) -> Self::Result {
        // Check user information
        let session_id = get_session_id_from_auth!(model);

        let user_type = match self.states.get_users_room_type(session_id, &model.room)? {
            Some(room_user_type) => room_user_type,
            None => return Err(anyhow::anyhow!(RoomError::UserNotInTheRoom))
        };

        if user_type == RoomUserType::User {
            return Err(anyhow::anyhow!(RoomError::UserDoesNotHaveEnoughPermission));
        }

        let users = self.states.get_join_requests(&model.room)?;
        model.socket.send(GenericAnswer::success(model.request_id, RoomResponse::WaitingRoomJoins { room: &model.room, users }).into());
        Ok(())
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<JoinToRoomRequest> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="JoinToRoom", skip(self, _ctx))]
    #[yummy_macros::plugin_api(name="join_to_room")]
    fn handle(&mut self, model: JoinToRoomRequest, _ctx: &mut Context<Self>) -> Self::Result {        
        // Check user information
        let (user_id, session_id) = get_user_session_id_from_auth!(model);

        let room_infos = self.states.get_room_info(&model.room, RoomMetaAccess::System, &[RoomInfoTypeVariant::JoinRequest])?;
        let join_require_approvement = room_infos.get_join_request();
        let mut connection = self.database.get()?;

        if self.states.is_user_banned_from_room(&model.room, user_id)? {
            return Err(anyhow::anyhow!(RoomError::BannedFromRoom));
        }

        if join_require_approvement.into_owned() {

            /* Room require approvement before join to it */
            self.states.join_to_room_request(&model.room, user_id, session_id, model.room_user_type.clone())?;

            // Save to database
            DB::join_to_room_request(&mut connection, &model.room, user_id, model.room_user_type.clone())?;

            let room_infos = self.states.get_room_info(&model.room, RoomMetaAccess::System, &[RoomInfoTypeVariant::Users])?;
            let users = room_infos.get_users();

            let message: String = RoomResponse::NewJoinRequest { room: &model.room, user: user_id, user_type: model.room_user_type.clone() }.into();
            
            for user in users.iter() {
                if user.user_type == RoomUserType::Owner || user.user_type == RoomUserType::Moderator {
                    self.issue_system_async(SendMessage {
                        message: message.clone(),
                        user_id: user.user_id.clone()
                    });
                }
            }

            // Send message to user about waiting for approvement
            model.socket.send(GenericAnswer::success(model.request_id, RoomResponse::JoinRequested { room: &model.room }).into());
        } else {
            // User can directly try to join room
            self.join_to_room(&mut connection,  model.request_id, &model.room, user_id, session_id, model.room_user_type.clone())?;
        }
        Ok(())
    }
}


impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<ProcessWaitingUser> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="ProcessWaitingUser", skip(self, _ctx))]
    #[yummy_macros::plugin_api(name="process_waiting_user")]
    fn handle(&mut self, model: ProcessWaitingUser, _ctx: &mut Context<Self>) -> Self::Result {        
        // Check user information
        let user_id = get_user_id_from_auth!(model);
        let (session_id, room_user_type) = self.states.remove_user_from_waiting_list(&model.user, &model.room)?;
        let mut connection = self.database.get()?;

        DB::transaction(&mut connection, |connection| {
            DB::update_join_to_room_request(connection, &model.room, &model.user, user_id, model.status)?;
            
            if model.status {

                // Moderator or room owner approve join request
                self.join_to_room(connection, model.request_id, &model.room, &model.user, &session_id, room_user_type)?;
            } else {
                
                // Room join request declined
                self.issue_system_async(SendMessage {
                    user_id: Arc::new(model.user.clone()),
                    message: GenericAnswer::success(model.request_id, RoomResponse::JoinRequestDeclined { room: &model.room }).into()
                });
            }

            // Send operation successfully executed message to operator
            model.socket.send(Answer::success(model.request_id).into());
            anyhow::Ok(())
        })?;
        
        Ok(())
    }
}


impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<KickUserFromRoom> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="KickUserFromRoom", skip(self, _ctx))]
    #[yummy_macros::plugin_api(name="kick_user_from_room")]
    fn handle(&mut self, model: KickUserFromRoom, _ctx: &mut Context<Self>) -> Self::Result {        
        let (user_id, session_id) = get_user_session_id_from_auth!(model);

        let requester_user_type = self.states.get_users_room_type(session_id, &model.room)?.ok_or(RoomError::UserDoesNotHaveEnoughPermission)?;

        // User must be room owner or moderator
        if requester_user_type == RoomUserType::User {
            return Err(anyhow::anyhow!(RoomError::UserDoesNotHaveEnoughPermission));
        }

        let session_id = self.states.get_user_session_id(&model.user, &model.room)?;
        
        // Disconnect user and send message to other users
        self.disconnect_from_room(&model.room, &model.user, &session_id)?;

        if model.ban {
            
            // Update state
            self.states.ban_user_from_room(&model.room, &model.user)?;

            // Update database
            let mut connection = self.database.get()?;
            DB::ban_user_from_room(&mut connection, &model.room, &model.user, user_id)?;
        }

        // Send message to use about disconnected from room
        if let Ok(message) = serde_json::to_string(&RoomResponse::DisconnectedFromRoom {
            room: &model.room
        }) {
            self.issue_system_async(SendMessage {
                message,
                user_id: Arc::new(model.user.clone())
            })
        }

        model.socket.send(Answer::success(model.request_id).into());
        Ok(())
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<DisconnectFromRoomRequest> for RoomManager<DB> {
    type Result = ();

    #[tracing::instrument(name="DisconnectFromRoomRequest", skip(self, _ctx))]
    #[yummy_macros::plugin_api(name="disconnect_from_room_request", no_return=true)]
    fn handle(&mut self, model: DisconnectFromRoomRequest, _ctx: &mut Context<Self>) -> Self::Result {

        #[allow(clippy::unused_unit)]
        let (user_id, session_id) = get_user_session_id_from_auth!(model, ());

        self.disconnect_from_room(&model.room, user_id, session_id).unwrap_or_default();
        model.socket.send(Answer::success(model.request_id).into());
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<MessageToRoomRequest> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="MessageToRoomRequest", skip(self, _ctx))]
    #[yummy_macros::plugin_api(name="message_to_room_request")]
    fn handle(&mut self, model: MessageToRoomRequest, _ctx: &mut Context<Self>) -> Self::Result {
        let sender_user_id = match model.auth.deref() {
            Some(user) => &user.user,
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        };

        match self.states.get_users_from_room(&model.room) {
            Ok(users) => {
                let message: String = RoomResponse::MessageFromRoom { user: sender_user_id, room: &model.room, message: &model.message }.into();

                for receiver_user in users.into_iter() {
                    if receiver_user.as_ref() != sender_user_id {
                        self.issue_system_async(SendMessage {
                            message: message.clone(),
                            user_id: receiver_user
                        });
                    }
                }

                model.socket.send(Answer::success(model.request_id).into());
                Ok(())
            }
            Err(error) => Err(anyhow!(error))
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<RoomListRequest> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="RoomListRequest", skip(self, _ctx))]
    #[yummy_macros::plugin_api(name="room_list_request")]
    fn handle(&mut self, model: RoomListRequest, _ctx: &mut Context<Self>) -> Self::Result {
        let members = if model.members.is_empty() {
            &ALL_ROOM_INFO_TYPE_VARIANTS
        } else {
            &model.members[..]
        };

        let rooms = self.states.get_rooms(&model.tag, members)?;
        model.socket.send(GenericAnswer::success(model.request_id, RoomResponse::RoomList { rooms }).into());
        Ok(())
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<GetRoomRequest> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="GetRoomRequest", skip(self, _ctx))]
    #[yummy_macros::plugin_api(name="get_room_request")]
    fn handle(&mut self, model: GetRoomRequest, _ctx: &mut Context<Self>) -> Self::Result {
        
        // Check user information
        let (user_id, session_id) = get_user_session_id_from_auth!(model);

        let members = if model.members.is_empty() {
            &ALL_ROOM_INFO_TYPE_VARIANTS
        } else {
            &model.members[..]
        };

        let access_level = self.get_access_level_for_room(user_id, session_id, &model.room)?;
        let room = self.states.get_room_info(&model.room, access_level, members)?;
        model.socket.send(GenericAnswer::success(model.request_id, RoomResponse::RoomInfo { room }).into());
        Ok(())
    }
}

/* **************************************************************************************************************** */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */
/* **************************************************************************************************************** */
