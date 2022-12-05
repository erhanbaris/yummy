pub mod model;

#[cfg(test)]
mod test;

use std::{marker::PhantomData, ops::Deref};
use std::sync::Arc;
use anyhow::Ok;

use actix::{Context, Actor, Handler};
use actix_broker::BrokerSubscribe;
use database::{Pool, DatabaseTrait};
use database::RowId;

use general::config::YummyConfig;
use general::model::{YummyState, RoomUserType, RoomId};
use rand::Rng;

use crate::response::Response;
use crate::api::auth::model::AuthError;

use self::model::*;

use super::auth::model::UserDisconnectRequest;

pub struct RoomManager<DB: DatabaseTrait + ?Sized> {
    config: Arc<YummyConfig>,
    database: Arc<Pool>,
    states: Arc<YummyState>,
    _marker: PhantomData<DB>,
}

impl<DB: DatabaseTrait + ?Sized> RoomManager<DB> {
    pub fn new(config: Arc<YummyConfig>, states: Arc<YummyState>, database: Arc<Pool>) -> Self {
        Self {
            config,
            database,
            states,
            _marker: PhantomData,
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

    #[tracing::instrument(name="User disconnected", skip(self, _ctx))]
    fn handle(&mut self, user: UserDisconnectRequest, _ctx: &mut Self::Context) -> Self::Result {
        println!("room:UserDisconnectRequest {:?}", user);
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<CreateRoomRequest> for RoomManager<DB> {
    type Result = anyhow::Result<Response>;

    #[tracing::instrument(name="Auth::ViaEmail", skip(self, _ctx))]
    fn handle(&mut self, model: CreateRoomRequest, _ctx: &mut Context<Self>) -> Self::Result {
        let CreateRoomRequest { access_type, disconnect_from_other_room, max_user, name, tags, user } = model;
        
        let user_id = match user.deref() {
            Some(user) => user.user.get(),
            None => return Err(anyhow::anyhow!(AuthError::TokenNotValid))
        };

        let mut connection = self.database.get()?;

        let mut randomizer = rand::thread_rng();
        let password: String = (0..self.config.room_password_length)
            .map(|_| {
                let idx = randomizer.gen_range(0..self.config.room_password_charset.len());
                self.config.room_password_charset[idx] as char
            })
            .collect();

        let room_id = DB::transaction(&mut connection, move |connection| {
            let room_id = DB::create_room(connection, name, access_type, Some(&password[..]), max_user, tags)?;
            DB::join_to_room(connection, room_id, RowId(user_id), RoomUserType::Owner)?;
            Ok(RoomId::from(room_id.get()))
        })?;

        Ok(Response::RoomInformation(room_id))
    }
}
