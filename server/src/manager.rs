use std::marker::PhantomData;
use std::fmt::Debug;

use actix::{Actor, Context, Handler, Recipient};
use actix::prelude::Message;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::auth::UserJwt;
use crate::model::{SessionId, RoomId};
use crate::websocket::WebsocketMessage;

#[derive(Message)]
#[rtype(result = "SessionId")]
pub struct Connect {
    pub connection_id: SessionId,
    pub user: UserJwt,
    pub socket: Box<Recipient<WebsocketMessage>>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartDisconnectProcedure {
    pub connection_id: SessionId,
    pub apply_now: bool,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UserMessage {
    pub connection_id: SessionId,
    pub message: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "join_type", content = "content")]
pub enum JoinToRoomType {
    Available,
    RoomId(RoomId),
    RoomIdAndSecret { room_id: RoomId, secret: u32 },
    Secret(u32),
}

#[derive(Deserialize, Serialize, Debug)]
pub enum NewGameType {
    Public,
    Private,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinToRoom {
    pub connection_id: SessionId,
    pub room: JoinToRoomType,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct KickPlayerFromRoom {
    pub connection_id: SessionId,
    pub room_id: RoomId,
}

#[derive(Message)]
#[rtype(result = "RoomId")]
pub struct NewGame<C>
where
    C: Debug + Serialize,
{
    pub connection_id: SessionId,
    pub game_type: NewGameType,
    pub config: Option<C>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ExitFromRoom {
    pub connection_id: SessionId,
}

#[derive(Message)]
#[rtype(result = "Vec<R>")]
pub struct RoomList<R: 'static>
where
    R: Debug + Serialize,
{
    pub _marker: PhantomData<R>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct MessageToRoom {
    pub connection_id: SessionId,
    pub message: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UserReady {
    pub connection_id: SessionId,
    pub room_id: RoomId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UserPlay<C>
where
    C: Debug + Serialize,
{
    pub connection_id: SessionId,
    pub room_id: RoomId,
    pub player_move: C,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UserAction<C>
where
    C: Debug + Serialize,
{
    pub connection_id: SessionId,
    pub room_id: RoomId,
    pub action: C,
}

pub trait GameManagerTrait: Actor<Context = Context<Self>> + Handler<Connect>
where
    Self: std::marker::Sized,
{
    type ConnectResponse: Serialize + DeserializeOwned + Send + Debug;
}
