use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::{fmt::Debug, borrow::Borrow};

use actix::{Recipient, Message};
use serde::de::DeserializeOwned;
use thiserror::Error;
use serde::Serialize;

#[cfg(feature = "stateless")]
use redis::Commands;

use crate::client::ClientTrait;
use crate::config::YummyConfig;
use crate::model::{UserId, RoomId, SessionId};

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct SendMessage {
    pub user_id: UserId,
    pub message: String
}

impl SendMessage {
    pub fn create<T:  Borrow<T> + Debug + Serialize + DeserializeOwned>(user_id: UserId, message: T) -> SendMessage {
        let message = serde_json::to_string(message.borrow());
        Self { user_id, message: message.unwrap_or_default() }
    }
}

#[derive(Clone)]
pub struct YummyState {
    config: Arc<YummyConfig>,

    // Fields for statefull informations
    #[cfg(not(feature = "stateless"))]
    user: Arc<Mutex<HashMap<UserId, UserState>>>,

    #[cfg(not(feature = "stateless"))]
    room: Arc<Mutex<HashMap<RoomId, RoomState>>>,
    
    #[cfg(not(feature = "stateless"))]
    session_to_user: Arc<Mutex<HashMap<SessionId, UserId>>>,

    // Fields for stateless informations
    #[cfg(feature = "stateless")]
    redis: r2d2::Pool<redis::Client>,

    #[cfg(feature = "stateless")]
    sender: Recipient<SendMessage>
}

impl YummyState {
    pub fn new(config: Arc<YummyConfig>, #[cfg(feature = "stateless")] redis: r2d2::Pool<redis::Client>, #[cfg(feature = "stateless")] sender: Recipient<SendMessage>) -> Self {
        Self {
            config,

            #[cfg(not(feature = "stateless"))] user: Arc::new(Mutex::default()),
            #[cfg(not(feature = "stateless"))] room: Arc::new(Mutex::default()),
            #[cfg(not(feature = "stateless"))] session_to_user: Arc::new(Mutex::default()),
            
            #[cfg(feature = "stateless")] redis,
            #[cfg(feature = "stateless")] sender
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum YummyStateError {
    #[error("Room not found")]
    RoomNotFound,
    
    #[error("User already in room")]
    UserAlreadInRoom,
    
    #[error("User could not found in the room")]
    UserCouldNotFoundInRoom,
    
    #[error("Room has max users")]
    RoomHasMaxUsers
}

impl YummyState {

    /* STATEFULL functions */
    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="is_user_online", skip(self))]
    pub fn is_user_online<T: Borrow<UserId> + std::fmt::Debug>(&mut self, user_id: T) -> bool {
        match self.redis.get() {
            Ok(mut redis) => redis.sismember(format!("{}online-users", self.config.redis_prefix), user_id.borrow().get().to_string()).unwrap_or_default(),
            Err(_) => false
        }
    }


    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="is_session_online", skip(self))]
    pub fn is_session_online<T: Borrow<SessionId> + std::fmt::Debug>(&mut self, session_id: T) -> bool {
        match self.redis.get() {
            Ok(mut redis) => redis.hexists::<_, _, bool>(format!("{}session-user", self.config.redis_prefix), session_id.borrow().get().to_string()).unwrap_or_default(),
            Err(_) => false
        }
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="new_session", skip(self))]
    pub fn new_session(&mut self, user_id: UserId, _: Arc<dyn ClientTrait + Sync + Send>) -> SessionId {
        let session_id = SessionId::new();
        match self.redis.get() {
            Ok(mut redis) => {
                redis.sadd::<_, _, i32>(format!("{}online-users", self.config.redis_prefix), user_id.borrow().get().to_string()).unwrap_or_default();
                redis.hset::<_, _, _, i32>(format!("{}session-user", self.config.redis_prefix), session_id.clone().get().to_string(), user_id.borrow().get().to_string()).unwrap_or_default();
                redis.hset::<_, _, _, i32>(format!("{}user-loc", self.config.redis_prefix), user_id.clone().get().to_string(), self.config.server_name.clone()).unwrap_or_default();
            },
            Err(_) => ()
        };
        session_id
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="close_session", skip(self))]
    pub fn close_session<T: Borrow<SessionId> + std::fmt::Debug>(&mut self, session_id: T) -> bool {
        match self.redis.get() {
            Ok(mut redis) => {
                let user_id = redis.hget::<_, _, String>(format!("{}session-user", self.config.redis_prefix), session_id.borrow().get().to_string()).unwrap_or_default();
                redis.hdel::<_, _, i32>(format!("{}session-user", self.config.redis_prefix), session_id.borrow().get().to_string()).unwrap_or_default();
                redis.hdel::<_, _, i32>(format!("{}user-room", self.config.redis_prefix), session_id.borrow().get().to_string()).unwrap_or_default();
                redis.srem::<_, _, i32>(format!("{}user-loc", self.config.redis_prefix), &user_id).unwrap_or_default();
                redis.srem::<_, _, i32>(format!("{}online-users", self.config.redis_prefix), user_id).unwrap_or_default() > 0
            },
            Err(_) => false
        }
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="get_user_room", skip(self))]
    pub fn get_user_room<T: Borrow<UserId> + std::fmt::Debug>(&mut self, user_id: T) -> Option<RoomId> {
        use std::str::FromStr;
        let result = match self.redis.get() {
            Ok(mut redis) => redis.hget::<_, _, String>(format!("{}user-room", self.config.redis_prefix), user_id.borrow().clone().get().to_string()).unwrap_or_default(),
            Err(_) => return None
        };
        
        match uuid::Uuid::from_str(&result) {
            Ok(item) => Some(RoomId::from(item)),
            Err(_) => None
        }
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="get_user_socket", skip(self))]
    pub fn get_user_socket<T: Borrow<UserId> + std::fmt::Debug>(&mut self, user_id: T) -> Option<Arc<dyn ClientTrait + Sync + Send>> {
        use crate::client::EmptyClient;

        Some(Arc::new(EmptyClient::default()))
        //self.user.lock().get(user_id.borrow()).map(|user| user.socket.clone())
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="set_user_room", skip(self))]
    pub fn set_user_room<T: Borrow<UserId> + std::fmt::Debug>(&mut self, user_id: T, room_id: RoomId) {
        if let Ok(mut redis) = self.redis.get() {
            redis.hset::<_, _, _, i32>(format!("{}user-room", self.config.redis_prefix), user_id.borrow().clone().get().to_string(), room_id.get().to_string()).unwrap_or_default();
        }
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="create_room", skip(self))]
    pub fn create_room(&mut self, room_id: RoomId, max_user: usize) {

        match self.redis.get() {
            Ok(mut redis) => {
                redis::cmd(&format!("{}HSET", self.config.redis_prefix))
                    .arg(format!("{}room-info:{}", self.config.redis_prefix, room_id.get().to_string()))
                    .arg("max-user").arg(max_user)
                    .arg("user-len").arg(0)
                    .execute(&mut redis)
            },
            Err(_) => ()
        };
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="join_to_room", skip(self))]
    pub fn join_to_room(&mut self, room_id: RoomId, user_id: UserId, user_type: crate::model::RoomUserType) -> Result<(), YummyStateError> {

        let room_info_key = format!("{}room-info:{}", self.config.redis_prefix, room_id.get());
        match self.redis.get() {
            Ok(mut redis) => match redis.exists::<_, bool>(&room_info_key).unwrap_or_default() {
                true => {
                    let room_users_key = format!("{}room-users:{}", self.config.redis_prefix, room_id.get());
                    let user_id = user_id.borrow().get().to_string();
    
                    let room_info = redis.hgetall::<_, HashMap<String, i32>>(&room_info_key).unwrap_or_default();
                    let user_len = room_info.get("user-len").map(|item| *item).unwrap_or_default();
                    let max_user = room_info.get("max-user").map(|item| *item).unwrap_or_default();
    
                    // If the max_user 0 or lower than users count, add to room
                    if max_user == 0 || max_user > user_len {
                        let is_member = redis.sismember(&room_users_key, user_id.clone()).unwrap_or_default();
    
                        // User alread in the room
                        if is_member {
                            return Err(YummyStateError::UserAlreadInRoom);
                        }

                        redis.hincr::<_, _, _, i32>(&room_info_key, "user-len", 1).unwrap_or_default();
                        redis.sadd::<_, _, i32>(room_users_key, user_id.clone()).unwrap_or_default();
                        Ok(())
                    } else {
                        Err(YummyStateError::RoomHasMaxUsers)
                    }
                }
                false => Err(YummyStateError::RoomNotFound)
            },
            Err(_) => Err(YummyStateError::RoomNotFound)
        }
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="join_to_room", skip(self))]
    pub fn disconnect_from_room(&mut self, room_id: RoomId, user_id: UserId) -> Result<bool, YummyStateError> {
        let room_removed: bool = match self.redis.get() {
            Ok(mut redis) => {
                let room_info_key = format!("{}room-info:{}", self.config.redis_prefix, room_id.get());
                let room_users_key = &format!("{}room-users:{}", self.config.redis_prefix, room_id.get());

                redis.srem::<_, _, i32>(&room_users_key, user_id.get().to_string()).unwrap_or_default();
                redis.hincr::<_, _, _, i32>(&room_info_key, "user-len", -1).unwrap_or_default();

                let user_len = redis.hget::<_, _, i32>(&room_info_key, "user-len").unwrap_or_default();
                let no_user = user_len == 0;

                if no_user {
                    redis.del::<_, i32>(&room_users_key).unwrap_or_default();
                    redis.del::<_, i32>(&room_info_key).unwrap_or_default();
                }

                no_user
            },
            Err(_) => false
        };
        Ok(room_removed)
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="get_users_from_room", skip(self))]
    pub fn get_users_from_room(&mut self, room_id: RoomId) -> Result<Vec<UserId>, YummyStateError> {
        use std::str::FromStr;
        let users: HashSet<String> = match self.redis.get() {
            Ok(mut redis) => match redis.exists::<_, bool>(&format!("{}room-users:{}", self.config.redis_prefix, room_id.get())).unwrap_or_default() {
                true => redis.smembers(&format!("{}room-users:{}", self.config.redis_prefix, room_id.get())).unwrap_or_default(),
                false => return Err(YummyStateError::RoomNotFound),
            },
            Err(_) => HashSet::default()
        };
        Ok(users.into_iter().map(|item| UserId::from(uuid::Uuid::from_str(&item[..]).unwrap_or_default())).collect::<Vec<UserId>>())
    }

    /* STATEFULL functions */
    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="is_user_online", skip(self))]
    pub fn is_user_online<T: Borrow<UserId> + std::fmt::Debug>(&self, user_id: T) -> bool {
        self.user.lock().contains_key(user_id.borrow())
    }
    
    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="is_session_online", skip(self))]
    pub fn is_session_online<T: Borrow<SessionId> + std::fmt::Debug>(&self, session_id: T) -> bool {
        self.session_to_user.lock().contains_key(session_id.borrow())
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="new_session", skip(self))]
    pub fn new_session(&self, user_id: UserId, socket: Arc<dyn ClientTrait + Sync + Send>) -> SessionId {
        use std::cell::Cell;

        let session_id = SessionId::new();
        self.session_to_user.lock().insert(session_id.clone(), user_id);
        self.user.lock().insert(user_id, UserState { user_id, session: session_id.clone(), room: Cell::new(None), socket });
        session_id
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="close_session", skip(self))]
    pub fn close_session<T: Borrow<SessionId> + std::fmt::Debug>(&self, session_id: T) -> bool {
        let removed = self.session_to_user.lock().remove(session_id.borrow());

        match removed {
            Some(removed) => self.user.lock().remove(&removed).map(|_| true).unwrap_or_default(),
            None => false
        }
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="get_user_room", skip(self))]
    pub fn get_user_room<T: Borrow<UserId> + std::fmt::Debug>(&self, user_id: T) -> Option<RoomId> {
        self.user.lock().get(user_id.borrow()).and_then(|user| user.room.get())
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="get_user_socket", skip(self))]
    pub fn get_user_socket<T: Borrow<UserId> + std::fmt::Debug>(&self, user_id: T) -> Option<Arc<dyn ClientTrait + Sync + Send>> {
        self.user.lock().get(user_id.borrow()).map(|user| user.socket.clone())
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="set_user_room", skip(self))]
    pub fn set_user_room<T: Borrow<UserId> + std::fmt::Debug>(&self, user_id: T, room_id: RoomId){
        if let Some(user) = self.user.lock().get(user_id.borrow()) {
            user.room.set(Some(room_id));
        }
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="create_room", skip(self))]
    pub fn create_room(&self, room_id: RoomId, max_user: usize) {
        self.room.lock().insert(room_id, RoomState { max_user, room_id, users: Mutex::new(HashSet::new()) });
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="join_to_room", skip(self))]
    pub fn join_to_room(&self, room_id: RoomId, user_id: UserId, user_type: RoomUserType) -> Result<(), YummyStateError> {

        // Get room
        match self.room.lock().get_mut(room_id.borrow()) {
            Some(room) => {
                let mut users = room.users.lock();
                let users_len = users.len();

                // If the max_user 0 or lower than users count, add to room
                if room.max_user == 0 || room.max_user > users_len {

                    // User alread in the room
                    if !users.insert(RoomUserInfo::new(user_id, user_type)) {
                        return Err(YummyStateError::UserAlreadInRoom);
                    }
                    Ok(())
                } else {
                    Err(YummyStateError::RoomHasMaxUsers)
                }
            }
            None => Err(YummyStateError::RoomNotFound)
        }
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="join_to_room", skip(self))]
    pub fn disconnect_from_room(&self, room_id: RoomId, user_id: UserId) -> Result<bool, YummyStateError> {
        let mut rooms = self.room.lock();
        let room_removed = match rooms.get_mut(room_id.borrow()) {
            Some(room) => {
                let mut users = room.users.lock();

                let user_removed = users.remove(&RoomUserInfo {
                    user_id,
                    room_user_type: RoomUserType::default() // Hash only consider user_id, so room_user_type not important in this case
                });

                match user_removed {
                    true => Ok(users.is_empty()),
                    false => Err(YummyStateError::UserCouldNotFoundInRoom)
                }
            }
            None => Err(YummyStateError::RoomNotFound)
        }?;

        if room_removed {
            rooms.remove(room_id.borrow());
        }

        Ok(room_removed)
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="get_users_from_room", skip(self))]
    pub fn get_users_from_room(&self, room_id: RoomId) -> Result<Vec<UserId>, YummyStateError> {
        match self.room.lock().get_mut(room_id.borrow()) {
            Some(room) => Ok(room.users.lock().iter().map(|item| item.user_id).collect::<Vec<_>>()),
            None => Err(YummyStateError::RoomNotFound)
        }
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{model::*, client::EmptyClient, config::get_configuration};

    #[cfg(feature = "stateless")]
    use crate::test::cleanup_redis;
    use actix::Actor;
    use actix::Context;
    use actix::Handler;
    use actix::Recipient;
    use anyhow::Ok;

    use super::SendMessage;
    use super::YummyState;
    use super::YummyStateError;

    struct DummyActor;
    impl Actor for DummyActor {
        type Context = Context<Self>;
    }
    
    impl Handler<SendMessage> for DummyActor {
        type Result = ();
    
        fn handle(&mut self, _: SendMessage, _ctx: &mut Self::Context) -> Self::Result {
        }
    }

    #[actix::test]
    async fn state_1() -> anyhow::Result<()> {
        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open("redis://127.0.0.1/").unwrap()).unwrap();

        #[cfg(feature = "stateless")]
        cleanup_redis(conn.clone());

        let config = get_configuration();

        let recipient = DummyActor{}.start().recipient::<SendMessage>();
        let mut state = YummyState::new(config, #[cfg(feature = "stateless")] conn, #[cfg(feature = "stateless")]  recipient);
        let user_id = UserId::new();
        let session_id = state.new_session(user_id, Arc::new(EmptyClient::default()));

        assert!(state.is_session_online(session_id.clone()));
        assert!(state.is_user_online(user_id.clone()));

        state.close_session(session_id.clone());

        assert!(!state.is_session_online(session_id.clone()));
        assert!(!state.is_user_online(user_id.clone()));

        Ok(())
    }

    #[actix::test]
    async fn state_2() -> anyhow::Result<()> {
        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open("redis://127.0.0.1/").unwrap()).unwrap();

        #[cfg(feature = "stateless")]
        cleanup_redis(conn.clone());

        let config = get_configuration();

        let recipient = DummyActor{}.start().recipient::<SendMessage>();
        let mut state = YummyState::new(config, #[cfg(feature = "stateless")] conn, #[cfg(feature = "stateless")]  recipient);
        
        state.close_session(SessionId::new());

        assert!(!state.is_session_online(SessionId::new()));
        assert!(!state.is_user_online(UserId::new()));

        Ok(())
    }
    
    #[actix::test]
    async fn room_tests() -> anyhow::Result<()> {
        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open("redis://127.0.0.1/").unwrap()).unwrap();

        #[cfg(feature = "stateless")]
        cleanup_redis(conn.clone());

        let config = get_configuration();

        let recipient = DummyActor{}.start().recipient::<SendMessage>();
        let mut state = YummyState::new(config, #[cfg(feature = "stateless")] conn, #[cfg(feature = "stateless")]  recipient);
        
        let room_1 = RoomId::new();
        state.create_room(room_1, 2);

        let user_1 = UserId::new();
        let user_2 = UserId::new();
        let user_3 = UserId::new();

        state.join_to_room(room_1.clone(), user_1.clone(), RoomUserType::Owner)?;
        assert_eq!(state.join_to_room(room_1.clone(), user_1.clone(), RoomUserType::Owner).err().unwrap(), YummyStateError::UserAlreadInRoom);

        state.join_to_room(room_1.clone(), user_2.clone(), RoomUserType::Owner)?;
        assert_eq!(state.join_to_room(room_1.clone(), user_3.clone(), RoomUserType::Owner).err().unwrap(), YummyStateError::RoomHasMaxUsers);
        assert_eq!(state.join_to_room(room_1.clone(), user_2.clone(), RoomUserType::Owner).err().unwrap(), YummyStateError::RoomHasMaxUsers);

        assert_eq!(state.join_to_room(RoomId::new(), UserId::new(), RoomUserType::Owner).err().unwrap(), YummyStateError::RoomNotFound);
        assert_eq!(state.get_users_from_room(room_1.clone())?.len(), 2);

        assert_eq!(state.disconnect_from_room(room_1.clone(), user_1.clone())?, false);
        assert_eq!(state.get_users_from_room(room_1.clone())?.len(), 1);

        assert_eq!(state.disconnect_from_room(room_1.clone(), user_2.clone())?, true);
        assert!(state.get_users_from_room(room_1.clone()).is_err());

        Ok(())
    }
    
    #[actix::test]
    async fn room_unlimited_users_tests() -> anyhow::Result<()> {
        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open("redis://127.0.0.1/").unwrap()).unwrap();

        #[cfg(feature = "stateless")]
        cleanup_redis(conn.clone());

        let config = get_configuration();

        let recipient = DummyActor{}.start().recipient::<SendMessage>();
        let mut state = YummyState::new(config, #[cfg(feature = "stateless")] conn, #[cfg(feature = "stateless")]  recipient);
    
        let room = RoomId::new();
        state.create_room(room, 0);

        for _ in 0..100_000 {
            state.join_to_room(room, UserId::new(), RoomUserType::Owner)?
        }

        Ok(())
    }
}
