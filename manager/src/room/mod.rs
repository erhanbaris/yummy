pub mod model;

#[cfg(test)]
mod test;

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{marker::PhantomData, ops::Deref};
use std::sync::Arc;
use actix::{Context, Actor, Handler};
use actix_broker::{BrokerSubscribe, BrokerIssue};
use anyhow::anyhow;
use database::model::RoomUpdate;
use database::{Pool, DatabaseTrait, PooledConnection};

use general::config::YummyConfig;
use general::meta::{MetaType, MetaAction};
use general::meta::RoomMetaAccess;
use general::model::{RoomId, UserId, RoomUserType, UserType};
use general::state::{YummyState, SendMessage, RoomInfoTypeVariant, RoomInfoType};
use general::web::{GenericAnswer, Answer};

use crate::auth::model::AuthError;
use crate::user::model::UserError;

use self::model::*;

use super::auth::model::UserDisconnectRequest;

type ConfigureMetasResult = anyhow::Result<(Option<HashMap<String, MetaType<RoomMetaAccess>>>, HashMap<String, MetaType<RoomMetaAccess>>)>;

pub struct RoomManager<DB: DatabaseTrait + ?Sized> {
    config: Arc<YummyConfig>,
    database: Arc<Pool>,
    states: YummyState,
    _marker: PhantomData<DB>
}

impl<DB: DatabaseTrait + ?Sized> RoomManager<DB> {
    pub fn new(config: Arc<YummyConfig>, states: YummyState, database: Arc<Pool>) -> Self {
        Self {
            config,
            database,
            states,
            _marker: PhantomData
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Actor for RoomManager<DB> {
    type Context = Context<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context) {
        self.subscribe_system_async::<UserDisconnectRequest>(ctx);
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<UserDisconnectRequest> for RoomManager<DB> {
    type Result = ();

    #[tracing::instrument(name="Room::User disconnected", skip(self, _ctx))]
    fn handle(&mut self, model: UserDisconnectRequest, _ctx: &mut Self::Context) -> Self::Result {
        println!("room:UserDisconnectRequest {:?}", model);
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> RoomManager<DB> {
    fn disconnect_from_room(&mut self, room_id: &RoomId, user_id: &UserId) -> anyhow::Result<bool> {
        let room_removed = self.states.disconnect_from_room(room_id, user_id)?;
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

    fn configure_metas(&self, connection: &mut PooledConnection, room_id: &RoomId, metas: Option<HashMap<String, MetaType<RoomMetaAccess>>>, meta_action: Option<MetaAction>, access_level: RoomMetaAccess) -> ConfigureMetasResult {
        let meta_action = meta_action.unwrap_or_default();
        let room_access_level_code = access_level.clone() as u8;

        let (to_be_inserted_metas, to_be_removed_metas, total_metas, remaining) = match meta_action {

            // Dont remove old metas
            general::meta::MetaAction::OnlyAddOrUpdate => {

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
            general::meta::MetaAction::RemoveUnusedMetas => {

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
            general::meta::MetaAction::RemoveAllMetas => {
                // Discard all new meta insertion list and remove all old meta that based on user access level.
                (None, Some(DB::get_room_meta(connection, room_id, access_level)?.into_iter().map(|meta| meta.0).collect::<Vec<_>>()), 0, HashMap::default())
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

    fn get_access_level_for_room(&mut self, user_id: &UserId, room_id: &RoomId) -> anyhow::Result<RoomMetaAccess> {
        match self.states.get_user_type(user_id) {
            Some(UserType::User) => match self.states.get_users_room_type(user_id, room_id) {
                Some(RoomUserType::User) => Ok(RoomMetaAccess::User),
                Some(RoomUserType::Moderator) => Ok(RoomMetaAccess::Moderator),
                Some(RoomUserType::Owner) => Ok(RoomMetaAccess::Owner),
                None => Err(anyhow::anyhow!(UserError::UserNotBelongToRoom))
            },
            Some(UserType::Mod) => Ok(RoomMetaAccess::Moderator),
            Some(UserType::Admin) => Ok(RoomMetaAccess::Admin),
            None => return Err(anyhow::anyhow!(UserError::UserNotFound))
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<CreateRoomRequest> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="CreateRoom", skip(self, _ctx))]
    #[macros::api(name="CreateRoom", socket=true)]
    fn handle(&mut self, model: CreateRoomRequest, _ctx: &mut Context<Self>) -> Self::Result {
        let CreateRoomRequest { access_type, max_user, name, description, tags, user, metas, join_request, socket } = model;
        
        // Check user information
        let user_id = match user.deref() {
            Some(user) => &user.user,
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        };

        // User already joined to room
        if let Some(room_id) = self.states.get_user_room(user_id) {
            self.disconnect_from_room(&room_id, user_id)?;
        }

        let mut connection = self.database.get()?;

        let room_id = DB::transaction(&mut connection, move |connection| {
            let insert_date = SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default();
            let room_id = DB::create_room(connection, name.clone(), access_type.clone(), max_user, join_request, &tags)?;

            DB::join_to_room(connection, &room_id, user_id, RoomUserType::Owner)?;

            let access_level = match self.states.get_user_type(user_id) {
                Some(UserType::User) => RoomMetaAccess::Owner,
                Some(UserType::Mod) => RoomMetaAccess::Owner,
                Some(UserType::Admin) => RoomMetaAccess::Admin,
                None => return Err(anyhow::anyhow!(UserError::UserNotFound))
            };
            
            let (mut meta, _) = self.configure_metas(connection, &room_id, metas, Some(MetaAction::OnlyAddOrUpdate), access_level)?;
            
            self.states.create_room(&room_id, insert_date, name, description, access_type, max_user, tags, meta, join_request);
            self.states.join_to_room(&room_id, user_id, RoomUserType::Owner)?;
           
            anyhow::Ok(room_id)
        })?;
        

        socket.send(GenericAnswer::success(RoomResponse::RoomCreated { room: room_id }).into());
        Ok(())
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<UpdateRoom> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="UpdateRoom", skip(self, _ctx))]
    #[macros::api(name="UpdateRoom", socket=true)]
    fn handle(&mut self, model: UpdateRoom, _ctx: &mut Context<Self>) -> Self::Result {

        let UpdateRoom { user, room_id, name, description, socket, metas, meta_action, access_type, max_user, tags, user_permission, join_request } = model;

        let user_id = match user.deref() {
            Some(user) => &user.user,
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        };

        let has_room_update = access_type.is_some() || max_user.is_some() || name.is_some() || description.is_some();

        if !has_room_update && metas.is_none() {
            return Err(anyhow::anyhow!(RoomError::UpdateInformationMissing));
        }

        // Calculate room access level for user
        let access_level = self.get_access_level_for_room(user_id, &room_id)?;

        let updates = RoomUpdate {
            max_user: max_user.map(|item| item as i32 ),
            access_type: access_type.map(|item| item.into() ),
            join_request: join_request.map(|item| item.into() ),
            name: name.map(|item| match item.trim().is_empty() { true => None, false => Some(item)} ),
            description: description.map(|item| match item.trim().is_empty() { true => None, false => Some(item)} )
        };

        let mut connection = self.database.get()?;

        DB::transaction::<_, anyhow::Error, _>(&mut connection, move |connection| {

            /* Meta configuration */
            let (mut metas, mut remaining) = self.configure_metas(connection, &model.room_id, metas, meta_action, access_level)?;

            /* Tag configuration */
            self.configure_tags(connection, &model.room_id, &tags)?;

            /* Change user permission */
            if let Some(user_permission) = user_permission {
                DB::update_room_user_permissions(connection, &model.room_id, &user_permission)?;
                
                for (user_id, user_type) in user_permission.into_iter() {
                    self.states.set_users_room_type(&user_id, &model.room_id, user_type);
                }
            }
            
            // Update user
            match has_room_update {
                true => match DB::update_room(connection, &model.room_id, &updates)? {
                    0 => return Err(anyhow::anyhow!(UserError::UserNotFound)),
                    _ => socket.send(Answer::success().into())
                },
                false => socket.send(Answer::success().into())
            };

            // Update all caches
            let mut room_update_query = Vec::new();
            if let Some(name) = updates.name {
                room_update_query.push(RoomInfoType::RoomName(name));
            }
            if let Some(description) = updates.description {
                room_update_query.push(RoomInfoType::Description(description));
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
                self.states.set_room_info(&room_id, room_update_query);
            }
            Ok(())
        })
    }
}

macro_rules! try_unpack {
    ($variant:path, $value:expr) => {
        if let $variant(x) = $value {
            Some(x)
        } else {
            None
        }   
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<JoinToRoomRequest> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="JoinToRoom", skip(self, _ctx))]
    #[macros::api(name="JoinToRoom", socket=true)]
    fn handle(&mut self, model: JoinToRoomRequest, _ctx: &mut Context<Self>) -> Self::Result {        
        // Check user information
        let user_id = match model.user.deref() {
            Some(user) => &user.user,
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        };

        if let Some(room_id) = self.states.get_user_room(user_id) {
            // User already joined to room, disconnect
            self.disconnect_from_room(&room_id, user_id)?;
        }

        let users = self.states.get_users_from_room(&model.room)?;
        self.states.join_to_room(&model.room, user_id, model.room_user_type.clone())?;
        
        let mut connection = self.database.get()?;
        DB::join_to_room(&mut connection, &model.room, user_id, model.room_user_type)?;

        let message = serde_json::to_string(&RoomResponse::UserJoinedToRoom {
            user: user_id,
            room: &model.room
        }).unwrap();

        for user_id in users.into_iter() {
            self.issue_system_async(SendMessage {
                message: message.clone(),
                user_id
            });
        }


        let access_level = self.get_access_level_for_room(user_id, &model.room)?;
        
        let infos = self.states.get_room_info(&model.room, access_level, vec![RoomInfoTypeVariant::RoomName, RoomInfoTypeVariant::Users, RoomInfoTypeVariant::Metas])?;
        let room_name = infos.get_item(RoomInfoTypeVariant::RoomName).and_then(|p| try_unpack!(RoomInfoType::RoomName, p)).unwrap_or_default();
        let users = infos.get_item(RoomInfoTypeVariant::Users).and_then(|p| try_unpack!(RoomInfoType::Users, p)).unwrap_or_default();
        let metas = infos.get_item(RoomInfoTypeVariant::Metas).and_then(|p| try_unpack!(RoomInfoType::Metas, p)).unwrap_or_default();
        
        model.socket.send(GenericAnswer::success(RoomResponse::Joined { room_name, users, metas, room: &model.room }).into());

        Ok(())
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<DisconnectFromRoomRequest> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="DisconnectFromRoom", skip(self, _ctx))]
    #[macros::api(name="DisconnectFromRoom", socket=true)]
    fn handle(&mut self, model: DisconnectFromRoomRequest, _ctx: &mut Context<Self>) -> Self::Result {        
        let user_id = match model.user.deref() {
            Some(user) => &user.user,
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        };

        match self.states.get_user_room(user_id) {
            Some(room_id) => {
                self.disconnect_from_room(&room_id, user_id)?;
                model.socket.send(Answer::success().into());
            }
            None => return Err(anyhow::anyhow!(RoomError::RoomNotFound))
        }

        Ok(())
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<MessageToRoomRequest> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="MessageToRoomRequest", skip(self, _ctx))]
    #[macros::api(name="MessageToRoomRequest", socket=true)]
    fn handle(&mut self, model: MessageToRoomRequest, _ctx: &mut Context<Self>) -> Self::Result {   
        let MessageToRoomRequest { user, room, message, socket } = model;

        let sender_user_id = match user.deref() {
            Some(user) => &user.user,
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        };

        match self.states.get_users_from_room(&room) {
            Ok(users) => {
                let message: String = RoomResponse::MessageFromRoom { user: sender_user_id, room: &model.room, message: Arc::new(message) }.into();

                for receiver_user in users.into_iter() {
                    if receiver_user.as_ref() != sender_user_id {
                        self.issue_system_async(SendMessage {
                            message: message.clone(),
                            user_id: receiver_user
                        });
                    }
                }

                socket.send(Answer::success().into());
                Ok(())
            }
            Err(error) => Err(anyhow!(error))
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<RoomListRequest> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="RoomListRequest", skip(self, _ctx))]
    #[macros::api(name="RoomListRequest", socket=true)]
    fn handle(&mut self, model: RoomListRequest, _ctx: &mut Context<Self>) -> Self::Result {
        let members = if model.members.is_empty() {
            vec![RoomInfoTypeVariant::Tags, RoomInfoTypeVariant::InsertDate, RoomInfoTypeVariant::RoomName, RoomInfoTypeVariant::AccessType, RoomInfoTypeVariant::Users, RoomInfoTypeVariant::MaxUser, RoomInfoTypeVariant::UserLength]
        } else {
            model.members
        };

        let rooms = self.states.get_rooms(model.tag, members)?;
        model.socket.send(GenericAnswer::success(RoomResponse::RoomList { rooms }).into());
        Ok(())
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<GetRoomRequest> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="GetRoomRequest", skip(self, _ctx))]
    #[macros::api(name="GetRoomRequest", socket=true)]
    fn handle(&mut self, model: GetRoomRequest, _ctx: &mut Context<Self>) -> Self::Result {
        
        // Check user information
        let user_id = match model.user.deref() {
            Some(user) => &user.user,
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        };

        let members = if model.members.is_empty() {
            vec![RoomInfoTypeVariant::Tags, RoomInfoTypeVariant::Metas, RoomInfoTypeVariant::InsertDate, RoomInfoTypeVariant::RoomName, RoomInfoTypeVariant::AccessType, RoomInfoTypeVariant::Users, RoomInfoTypeVariant::MaxUser, RoomInfoTypeVariant::UserLength]
        } else {
            model.members
        };

        let access_level = self.get_access_level_for_room(user_id, &model.room)?;
        let room = self.states.get_room_info(&model.room, access_level, members)?;
        model.socket.send(GenericAnswer::success(RoomResponse::RoomInfo { room }).into());
        Ok(())
    }
}
