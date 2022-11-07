use std::marker::PhantomData;
use std::fmt::Debug;

use actix::{Actor, Context, Handler};
use actix::prelude::Message;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::auth::UserJwt;
use crate::model::{ConnectionId, RoomId};
use crate::websocket::socket::SocketTrait;

#[derive(Message)]
#[rtype(result = "ConnectionId")]
pub struct Connect<R>
where
    R: Debug + Serialize,
{
    pub connection_id: ConnectionId,
    pub user: UserJwt,
    pub socket: Box<dyn SocketTrait<Message = R> + Send>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartDisconnectProcedure {
    pub connection_id: ConnectionId,
    pub apply_now: bool,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UserMessage {
    pub connection_id: ConnectionId,
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
    pub connection_id: ConnectionId,
    pub room: JoinToRoomType,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct KickPlayerFromRoom {
    pub connection_id: ConnectionId,
    pub room_id: RoomId,
}

#[derive(Message)]
#[rtype(result = "RoomId")]
pub struct NewGame<C>
where
    C: Debug + Serialize,
{
    pub connection_id: ConnectionId,
    pub game_type: NewGameType,
    pub config: Option<C>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ExitFromRoom {
    pub connection_id: ConnectionId,
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
    pub connection_id: ConnectionId,
    pub message: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UserReady {
    pub connection_id: ConnectionId,
    pub room_id: RoomId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UserPlay<C>
where
    C: Debug + Serialize,
{
    pub connection_id: ConnectionId,
    pub room_id: RoomId,
    pub player_move: C,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UserAction<C>
where
    C: Debug + Serialize,
{
    pub connection_id: ConnectionId,
    pub room_id: RoomId,
    pub action: C,
}

pub trait GameManagerTrait:
    Actor<Context = Context<Self>>
    + Handler<Connect<Self::ConnectResponse>>
where
    Self: std::marker::Sized,
{
    type ConnectResponse: Serialize + DeserializeOwned + Send + Debug;
}
