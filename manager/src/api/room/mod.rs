pub mod model;

#[cfg(test)]
mod test;

use std::{marker::PhantomData, ops::Deref};
use std::sync::Arc;
use anyhow::Ok;

use actix::{Context, Actor, Handler};
use actix_broker::{BrokerSubscribe};
use database::{Pool, DatabaseTrait};
use database::RowId;

use general::config::YummyConfig;
use general::model::{YummyState, RoomUserType, RoomId, UserId};
use general::web::GenericAnswer;
use rand::Rng;

use crate::api::auth::model::AuthError;

use self::model::*;

use super::auth::model::UserDisconnectRequest;

pub struct RoomManager<DB: DatabaseTrait + ?Sized> {
    config: Arc<YummyConfig>,
    database: Arc<Pool>,
    states: Arc<YummyState>,
    _marker: PhantomData<DB>
}

impl<DB: DatabaseTrait + ?Sized> RoomManager<DB> {
    pub fn new(config: Arc<YummyConfig>, states: Arc<YummyState>, database: Arc<Pool>) -> Self {
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
    
    fn started(&mut self,ctx: &mut Self::Context) {
        self.subscribe_system_async::<UserDisconnectRequest>(ctx);
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<UserDisconnectRequest> for RoomManager<DB> {
    type Result = ();

    #[tracing::instrument(name="Room::User disconnected", skip(self, _ctx))]
    fn handle(&mut self, user: UserDisconnectRequest, _ctx: &mut Self::Context) -> Self::Result {
        println!("room:UserDisconnectRequest {:?}", user);
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> RoomManager<DB> {
    pub fn disconnect_from_room(&self, room_id: RoomId, user_id: UserId) -> anyhow::Result<bool> {
        let room_removed = self.states.disconnect_from_room(room_id.clone(), user_id.clone())?;
        let users = self.states.get_users_from_room(room_id.clone())?;
        
        let mut connection = self.database.get()?;
        DB::disconnect_from_room(&mut connection, RowId(room_id.get()), RowId(user_id.get()))?;

        for user in users.into_iter() {
            if let Some(socket) = self.states.get_user_socket(user) {
                socket.send(serde_json::to_string(&RoomResponse::UserDisconnectedFromRoom {
                    user: user_id,
                    room: room_id
                }).unwrap_or_default());
            }
        }

        Ok(room_removed)
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<CreateRoomRequest> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="Room::Create room", skip(self, _ctx))]
    #[macros::api(name="ViaEmail", socket=true)]
    fn handle(&mut self, model: CreateRoomRequest, _ctx: &mut Context<Self>) -> Self::Result {
        let CreateRoomRequest { access_type, disconnect_from_other_room, max_user, name, tags, user, socket } = model;
        
        // Check user information
        let user_id = match user.deref() {
            Some(user) => UserId::from(user.user.get()),
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

        /* Create random password */
        let mut randomizer = rand::thread_rng();
        let password: String = (0..self.config.room_password_length)
            .map(|_| {
                let idx = randomizer.gen_range(0..self.config.room_password_charset.len());
                self.config.room_password_charset[idx] as char
            })
            .collect();

        let room_id = DB::transaction(&mut connection, move |connection| {
            let room_id = DB::create_room(connection, name, access_type, Some(&password[..]), max_user, tags)?;

            DB::join_to_room(connection, room_id, RowId(user_id.get()), RoomUserType::Owner)?;

            let room_id = RoomId::from(room_id.get());
            self.states.create_room(room_id, max_user);
            self.states.join_to_room(room_id, user_id, RoomUserType::Owner)?;
            self.states.set_user_room(user_id, room_id);

            Ok(room_id)
        })?;
        

        socket.send(GenericAnswer::success(room_id).into());
        Ok(())
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<JoinToRoomRequest> for RoomManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="Room::Join to room", skip(self, _ctx))]
    #[macros::api(name="ViaEmail", socket=true)]
    fn handle(&mut self, model: JoinToRoomRequest, _ctx: &mut Context<Self>) -> Self::Result {        
        // Check user information
        let user_id = match model.user.deref() {
            Some(user) => UserId::from(user.user.get()),
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        };

        if let Some(room_id) = self.states.get_user_room(user_id) {
            // User already joined to room, disconnect
            self.disconnect_from_room(room_id, user_id)?;
        }

        let users = self.states.get_users_from_room(model.room)?;
        self.states.join_to_room(model.room, user_id, model.room_user_type)?;
        
        let mut connection = self.database.get()?;
        DB::join_to_room(&mut connection, RowId(model.room.get()), RowId(user_id.get()), model.room_user_type)?;

        for user in users.into_iter() {
            if let Some(socket) = self.states.get_user_socket(user) {
                socket.send(serde_json::to_string(&RoomResponse::UserJoinedToRoom {
                    user: user_id,
                    room: model.room
                }).unwrap_or_default());
            }
        }

        Ok(())
    }
}
