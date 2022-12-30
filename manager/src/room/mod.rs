pub mod model;

#[cfg(test)]
mod test;

use std::time::{SystemTime, UNIX_EPOCH};
use std::{marker::PhantomData, ops::Deref};
use std::sync::Arc;
use actix::{Context, Actor, Handler};
use actix_broker::{BrokerSubscribe, BrokerIssue};
use anyhow::anyhow;
use database::{Pool, DatabaseTrait};

use general::config::YummyConfig;
use general::model::{RoomId, UserId, RoomUserType};
use general::state::{YummyState, SendMessage, RoomInfoTypeVariant, RoomInfoType};
use general::web::{GenericAnswer, Answer};

use crate::auth::model::AuthError;

use self::model::*;

use super::auth::model::UserDisconnectRequest;

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
    pub fn disconnect_from_room(&mut self, room_id: RoomId, user_id: UserId) -> anyhow::Result<bool> {
        let room_removed = self.states.disconnect_from_room(room_id, user_id)?;
        let users = self.states.get_users_from_room(room_id)?;
        
        let mut connection = self.database.get()?;
        DB::disconnect_from_room(&mut connection, room_id.into(), user_id.into())?;

        let message = serde_json::to_string(&RoomResponse::UserDisconnectedFromRoom {
            user: user_id,
            room: room_id
        }).unwrap_or_default();

        for user_id in users.into_iter() {
            self.issue_system_async(SendMessage {
                message: message.clone(),
                user_id
            });
        }

        Ok(room_removed)
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<CreateRoomRequest> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="CreateRoom", skip(self, _ctx))]
    #[macros::api(name="CreateRoom", socket=true)]
    fn handle(&mut self, model: CreateRoomRequest, _ctx: &mut Context<Self>) -> Self::Result {
        let CreateRoomRequest { access_type, disconnect_from_other_room, max_user, name, tags, user, meta, socket } = model;
        
        // Check user information
        let user_id = match user.deref() {
            Some(user) => user.user,
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        };

        // User already joined to room
        if let Some(room_id) = self.states.get_user_room(user_id) {
            match disconnect_from_other_room {
                true => self.disconnect_from_room(room_id, user_id)?,
                false => return Err(anyhow::anyhow!(RoomError::UserJoinedOtherRoom))
            };
        }

        let mut connection = self.database.get()?;

        let room_id = DB::transaction(&mut connection, move |connection| {
            let insert_date = SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default();
            let room_id = DB::create_room(connection, name.clone(), access_type.clone(), max_user, &tags)?;

            DB::join_to_room(connection, room_id.clone(), user_id.into(), RoomUserType::Owner)?; // todo: discart cloning

            if let Some(meta) = meta {
                DB::insert_metas(connection, room_id.clone(), meta.into_iter().map(|(key, value)| (key, value)).collect::<Vec<_>>())?; // todo: discart cloning
            }
            
            let room_id: RoomId = room_id.into();
            self.states.create_room(room_id, insert_date, name, access_type, max_user, tags);
            self.states.join_to_room(room_id, user_id, RoomUserType::Owner)?;
            self.states.set_user_room(user_id, room_id);

            anyhow::Ok(room_id)
        })?;
        

        socket.send(GenericAnswer::success(room_id).into());
        Ok(())
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<UpdateRoom> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="UpdateRoom", skip(self, _ctx))]
    #[macros::api(name="UpdateRoom", socket=true)]
    fn handle(&mut self, model: UpdateRoom, _ctx: &mut Context<Self>) -> Self::Result {
        /*let original_user_id = match model.user.deref() {
            Some(user) => user.user.get(),
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        };

        let has_room_update = model.access_type.is_some() || model.max_user.is_some() || model.name.is_some() || model.tags.is_some();

        if !has_room_update && model.meta.as_ref().map(|dict| dict.len()).unwrap_or_default() == 0 {
            return Err(anyhow::anyhow!(RoomError::UpdateInformationMissing));
        }

        let mut updates = RoomUpdate::default();
        let user_id = RowId(original_user_id);

        let mut connection = self.database.get()?;

        updates.max_user = model.max_user.map(|item| item as i32 );
        updates.access_type = model.access_type.map(|item| item.into() );
        updates.name = model.name.map(|item| match item.trim().is_empty() { true => None, false => Some(item)} );

        let mut connection = self.database.get()?;
        let config = self.config.clone();

        DB::transaction::<_, anyhow::Error, _>(&mut connection, move |connection| {
            if let Some(meta) = model.meta {

                match meta.len() {
                    0 => (),
                    n if n > config.max_room_meta => return Err(anyhow::anyhow!(RoomError::MetaLimitOverToMaximum)),
                    _ => {
                        let user_old_metas = DB::get_room_meta(connection, user_id, model.access_level)?;
                        let mut remove_list = Vec::new();
                        let mut insert_list = Vec::new();

                        for (key, value) in meta.into_iter() {
                            let row= user_old_metas.iter().find(|item| item.1 == key).map(|item| item.0);

                            /* Remove the key if exists in the database */
                            if let Some(row_id) = row {
                                remove_list.push(row_id);
                            }

                            /* Remove meta */
                            if let MetaType::<RoomMetaAccess>::Null = value {
                                continue;
                            }

                            insert_list.push((key, value));
                        }

                        DB::remove_room_metas(connection, remove_list)?;
                        DB::insert_room_metas(connection, model.room_id, insert_list)?;
                    }
                };
            }
            
            // Update user
            match has_room_update {
                true => match DB::update_user(connection, user_id, updates)? {
                    0 => Err(anyhow::anyhow!(RoomError::RoomNotFound)),
                    _ => {
                        model.socket.send(Answer::success().into());
                        Ok(())
                    }
                },
                false => {
                    model.socket.send(Answer::success().into());
                    Ok(())
                }
            }
        })*/
        Ok(())
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
            Some(user) => user.user,
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        };

        if let Some(room_id) = self.states.get_user_room(user_id) {
            // User already joined to room, disconnect
            self.disconnect_from_room(room_id, user_id)?;
        }

        let users = self.states.get_users_from_room(model.room)?;
        self.states.join_to_room(model.room, user_id, model.room_user_type.clone())?; // todo: discart cloning
        
        let mut connection = self.database.get()?;
        DB::join_to_room(&mut connection, model.room.into(), user_id.into(), model.room_user_type)?;

        let message = serde_json::to_string(&RoomResponse::UserJoinedToRoom {
            user: user_id,
            room: model.room
        }).unwrap_or_default();

        for user_id in users.into_iter() {
            self.issue_system_async(SendMessage {
                message: message.clone(),
                user_id
            });
        }

        let infos = self.states.get_room_info(model.room, vec![RoomInfoTypeVariant::RoomName, RoomInfoTypeVariant::Users])?;
        let room_name = infos.get_item(RoomInfoTypeVariant::RoomName).and_then(|p| try_unpack!(RoomInfoType::RoomName, p)).unwrap_or_default();
        let users = infos.get_item(RoomInfoTypeVariant::Users).and_then(|p| try_unpack!(RoomInfoType::Users, p)).unwrap_or_default();
        
        model.socket.send(GenericAnswer::success(RoomResponse::Joined { room_name, users }).into());

        Ok(())
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<DisconnectFromRoomRequest> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="DisconnectFromRoom", skip(self, _ctx))]
    #[macros::api(name="DisconnectFromRoom", socket=true)]
    fn handle(&mut self, model: DisconnectFromRoomRequest, _ctx: &mut Context<Self>) -> Self::Result {        
        let user_id = match model.user.deref() {
            Some(user) => user.user,
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        };

        match self.states.get_user_room(user_id) {
            Some(room_id) => {
                self.disconnect_from_room(room_id, user_id)?;
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
            Some(user) => user.user,
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        };

        match self.states.get_users_from_room(room) {
            Ok(users) => {
                let message: String = RoomResponse::MessageFromRoom { user: sender_user_id, room: model.room, message: Arc::new(message) }.into();

                for receiver_user in users.into_iter() {
                    if receiver_user != sender_user_id {
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

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<RoomListRequet> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="MessageToRoomRequest", skip(self, _ctx))]
    #[macros::api(name="MessageToRoomRequest", socket=true)]
    fn handle(&mut self, model: RoomListRequet, _ctx: &mut Context<Self>) -> Self::Result {
        let members = if model.members.is_empty() {
            vec![RoomInfoTypeVariant::Tags, RoomInfoTypeVariant::InsertDate, RoomInfoTypeVariant::RoomName, RoomInfoTypeVariant::AccessType, RoomInfoTypeVariant::Users, RoomInfoTypeVariant::MaxUser, RoomInfoTypeVariant::UserLength]
        } else {
            model.members
        };

        let rooms = self.states.get_rooms(model.tag, members)?;
        model.socket.send(GenericAnswer::success(rooms).into());
        Ok(())
    }
}
