use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::Arc;
use std::{fmt::Debug, borrow::Borrow};

use actix::Message;
use serde::de::DeserializeOwned;
use strum_macros::EnumDiscriminants;
use thiserror::Error;
use serde::{Serialize, Deserialize};

#[cfg(feature = "stateless")]
use redis::Commands;

use crate::config::YummyConfig;
use crate::model::{UserId, RoomId, SessionId};
use crate::model::RoomUserType;
use crate::model::UserState;
use crate::model::RoomState;
use crate::model::CreateRoomAccessType;
use parking_lot::Mutex;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct RoomUserInformation {
    pub user_id: UserId,
    pub name: Option<String>
}

#[derive(Message, Debug, Clone, Serialize, Deserialize)]
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


#[derive(Debug, Clone, EnumDiscriminants, PartialEq)]
#[strum_discriminants(name(RoomInfoTypeVariant))]
pub enum RoomInfoType {
    RoomName(Option<String>),
    Users(Vec<RoomUserInformation>),
    MaxUser(usize),
    UserLength(usize),
    Password(Option<String>)
}

#[derive(Debug, Default)]
pub struct RoomQueryResult {
    pub results: Vec<RoomInfoType>
}

impl RoomQueryResult {
    pub fn get_item(&self, query: RoomInfoTypeVariant) -> Option<RoomInfoType> {
        self.results.iter().find(|item| query == RoomInfoTypeVariant::from(item.deref())).cloned()
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
    redis: r2d2::Pool<redis::Client>
}

impl YummyState {
    pub fn new(config: Arc<YummyConfig>, #[cfg(feature = "stateless")] redis: r2d2::Pool<redis::Client>) -> Self {
        Self {
            config,

            #[cfg(not(feature = "stateless"))] user: Arc::new(Mutex::default()),
            #[cfg(not(feature = "stateless"))] room: Arc::new(Mutex::default()),
            #[cfg(not(feature = "stateless"))] session_to_user: Arc::new(Mutex::default()),
            
            #[cfg(feature = "stateless")] redis
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
    RoomHasMaxUsers,
    
    #[error("Cache could not readed")]
    CacheCouldNotReaded
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
    pub fn new_session(&mut self, user_id: UserId, name: Option<String>) -> SessionId {
        let session_id = SessionId::new();
        match self.redis.get() {
            Ok(mut redis) => {
                redis.sadd::<_, _, i32>(format!("{}online-users", self.config.redis_prefix), user_id.borrow().get().to_string()).unwrap_or_default();
                redis.hset::<_, _, _, i32>(format!("{}session-user", self.config.redis_prefix), session_id.clone().get().to_string(), user_id.borrow().get().to_string()).unwrap_or_default();

                redis::cmd("HSET")
                    .arg(format!("{}users:{}", self.config.redis_prefix, user_id.to_string()))
                    .arg("room").arg("")
                    .arg("name").arg(name.unwrap_or_default())
                    .arg("loc").arg(&self.config.server_name)
                    .query::<()>(&mut redis)
                    .unwrap_or_default();
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
                redis.del::<_, i32>(format!("{}users:{}", self.config.redis_prefix, user_id.to_string())).unwrap_or_default();
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
            Ok(mut redis) => redis.hget::<_, _, String>(format!("{}users:{}", self.config.redis_prefix, user_id.borrow().to_string()), "room").unwrap_or_default(),
            Err(_) => return None
        };
        
        match uuid::Uuid::from_str(&result) {
            Ok(item) => Some(RoomId::from(item)),
            Err(_) => None
        }
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="set_user_room", skip(self))]
    pub fn set_user_room<T: Borrow<UserId> + std::fmt::Debug>(&mut self, user_id: T, room_id: RoomId) {
        if let Ok(mut redis) = self.redis.get() {
            redis.hset::<_, _, _, i32>(format!("{}users:{}", self.config.redis_prefix, user_id.borrow().to_string()), "room", room_id.get().to_string()).unwrap_or_default();
        }
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="create_room", skip(self))]
    pub fn create_room(&self, room_id: RoomId, name: Option<String>, access_type: CreateRoomAccessType, password: Option<String>, max_user: usize, tags: Vec<String>) {
        match self.redis.get() {
            Ok(mut redis) => {
                redis::cmd("HSET")
                    .arg(format!("{}room:{}", self.config.redis_prefix, room_id.get().to_string()))
                    .arg("max-user").arg(max_user)
                    .arg("user-len").arg(0_usize)
                    .arg("name").arg(name.unwrap_or_default())
                    .arg("pass").arg(password.unwrap_or_default())
                    .execute(&mut redis)
            },
            Err(_) => ()
        };
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="join_to_room", skip(self))]
    pub fn join_to_room(&mut self, room_id: RoomId, user_id: UserId, user_type: crate::model::RoomUserType) -> Result<(), YummyStateError> {

        let room_info_key = format!("{}room:{}", self.config.redis_prefix, room_id.get());
        match self.redis.get() {
            Ok(mut redis) => match redis.exists::<_, bool>(&room_info_key).unwrap_or_default() {
                true => {
                    let room_users_key = format!("{}room-users:{}", self.config.redis_prefix, room_id.get());
                    let user_id = user_id.borrow().get().to_string();

                    let room_info = redis::cmd("HMGET")
                        .arg(format!("{}room:{}", self.config.redis_prefix, room_id.get().to_string()))
                        .arg("user-len")
                        .arg("max-user")
                        .query::<Vec<usize>>(&mut redis).unwrap_or_default();

                    let user_len = room_info.get(0).cloned().unwrap_or_default();
                    let max_user = room_info.get(1).cloned().unwrap_or_default();

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
                let room_info_key = format!("{}room:{}", self.config.redis_prefix, room_id.get());
                let room_users_key = &format!("{}room-users:{}", self.config.redis_prefix, room_id.get());

                redis.srem::<_, _, i32>(&room_users_key, user_id.to_string()).unwrap_or_default();
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

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="get_user_location", skip(self))]
    pub fn get_user_location(&self, user_id: UserId) -> Option<String> {
        match self.redis.get() {
            Ok(mut redis) => match redis.hget::<_, _, String>(format!("{}users:{}", self.config.redis_prefix, user_id.to_string()), "loc") {
                Ok(result) => Some(result),
                Err(_) => None
            },
            Err(_) => None
        }
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="get_room_info", skip(self))]
    pub fn get_room_info(&self, room_id: RoomId, query: Vec<RoomInfoTypeVariant>) -> Result<RoomQueryResult, YummyStateError> {
        use redis::FromRedisValue;

        let mut result = RoomQueryResult::default();
        if query.is_empty() {
            return Ok(result);
        }

        match self.redis.get() {
            Ok(mut redis) => {
                let mut command = redis::cmd("HMGET");
                let mut request = command.arg(format!("{}room:{}", self.config.redis_prefix, room_id.get().to_string()));
                
                for item in query.iter() {
                    request = match item {
                        RoomInfoTypeVariant::RoomName => request.arg("name"),
                        RoomInfoTypeVariant::Users => request.arg("users"),
                        RoomInfoTypeVariant::MaxUser => request.arg("max-user"),
                        RoomInfoTypeVariant::UserLength => request.arg("user-len"),
                        RoomInfoTypeVariant::Password => request.arg("pass"),
                    };
                }

                let room_infos = request.query::<Vec<redis::Value>>(&mut redis).unwrap_or_default();

                for (query, room_info) in query.into_iter().zip(room_infos.into_iter()) {
                    let result_item = match query {
                        RoomInfoTypeVariant::RoomName => {
                            let room_name: String = FromRedisValue::from_redis_value(&room_info).unwrap_or_default();
                            RoomInfoType::RoomName(if room_name.is_empty() { None } else { Some(room_name) })
                        },
                        RoomInfoTypeVariant::Users => {
                            let mut user_infos = Vec::new();
                            let users = redis.smembers::<_, Vec<String>>(format!("{}room-users:{}", self.config.redis_prefix, room_id.get())).unwrap_or_default();
                            for user_id in users.iter() {
                                let name = redis.hget::<_, _, String>(format!("{}users:{}", self.config.redis_prefix, user_id), "name").unwrap_or_default();
                                user_infos.push(RoomUserInformation {
                                    name: if name.is_empty() { None } else { Some(name) },
                                    user_id: UserId::from(uuid::Uuid::parse_str(user_id).unwrap_or_default())
                                })
                            }
                            RoomInfoType::Users(user_infos)
                        },
                        RoomInfoTypeVariant::MaxUser => RoomInfoType::MaxUser(FromRedisValue::from_redis_value(&room_info).unwrap_or_default()),
                        RoomInfoTypeVariant::UserLength => RoomInfoType::UserLength(FromRedisValue::from_redis_value(&room_info).unwrap_or_default()),
                        RoomInfoTypeVariant::Password => {
                            let password: String = FromRedisValue::from_redis_value(&room_info).unwrap_or_default();
                            RoomInfoType::Password(if password.is_empty() { None } else { Some(password) })
                        },
                    };

                    result.results.push(result_item);
                }

                Ok(result)
            },
            Err(_) => Err(YummyStateError::CacheCouldNotReaded)
        }
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
    pub fn new_session(&self, user_id: UserId, name: Option<String>) -> SessionId {
        use std::cell::Cell;

        let session_id = SessionId::new();
        self.session_to_user.lock().insert(session_id.clone(), user_id);
        self.user.lock().insert(user_id, UserState { user_id, name, session: session_id.clone(), room: Cell::new(None) });
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
    #[tracing::instrument(name="set_user_room", skip(self))]
    pub fn set_user_room<T: Borrow<UserId> + std::fmt::Debug>(&self, user_id: T, room_id: RoomId){
        if let Some(user) = self.user.lock().get(user_id.borrow()) {
            user.room.set(Some(room_id));
        }
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="create_room", skip(self))]
    pub fn create_room(&self, room_id: RoomId, name: Option<String>, access_type: CreateRoomAccessType, password: Option<String>, max_user: usize, tags: Vec<String>) {
        self.room.lock().insert(room_id, RoomState { max_user, room_id, users: Mutex::new(HashSet::new()), tags, name, access_type, password });
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="join_to_room", skip(self))]
    pub fn join_to_room(&self, room_id: RoomId, user_id: UserId, user_type: RoomUserType) -> Result<(), YummyStateError> {
        use crate::model::RoomUserInfo;
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
        use crate::model::{RoomUserType, RoomUserInfo};

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

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="get_user_location", skip(self))]
    pub fn get_user_location(&self, user_id: UserId) -> Option<String> {
        None
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="get_room_info", skip(self))]
    pub fn get_room_info(&self, room_id: RoomId, query: Vec<RoomInfoTypeVariant>) -> Result<RoomQueryResult, YummyStateError> {
        let mut result = RoomQueryResult::default();
        match self.room.lock().get(&room_id) {
            Some(room) => {
                for item in query.iter() {
                    let item = match item {
                        RoomInfoTypeVariant::MaxUser => RoomInfoType::MaxUser(room.max_user),
                        RoomInfoTypeVariant::Password => RoomInfoType::Password(room.password.clone()),
                        RoomInfoTypeVariant::RoomName => RoomInfoType::RoomName(room.name.clone()),
                        RoomInfoTypeVariant::UserLength => RoomInfoType::UserLength(room.users.lock().len()),
                        RoomInfoTypeVariant::Users => {
                            let  mut users = Vec::new();
                            for user in room.users.lock().iter() {
                                let name = match self.user.lock().get(&user.user_id) {
                                    Some(user) => user.name.clone(),
                                    None => None
                                };
                                users.push(RoomUserInformation {
                                    user_id: user.user_id,
                                    name
                                });
                            }

                            RoomInfoType::Users(users)
                        }
                    };
        
                    result.results.push(item);
                }

                Ok(result)
            },
            None => Err(YummyStateError::RoomNotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::configure_environment;
    use crate::{model::*, config::get_configuration};

    #[cfg(feature = "stateless")]
    use crate::test::cleanup_redis;
    use actix::Actor;
    use actix::Context;
    use actix::Handler;
    use anyhow::Ok;

    use super::*;

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
        configure_environment();
        let config = get_configuration();

        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();

        #[cfg(feature = "stateless")]
        cleanup_redis(conn.clone());

        DummyActor{}.start().recipient::<SendMessage>();
        let mut state = YummyState::new(config, #[cfg(feature = "stateless")] conn);
        let user_id = UserId::new();
        let session_id = state.new_session(user_id, None);

        assert!(state.is_session_online(session_id.clone()));
        assert!(state.is_user_online(user_id.clone()));

        state.close_session(session_id.clone());

        assert!(!state.is_session_online(session_id.clone()));
        assert!(!state.is_user_online(user_id.clone()));

        Ok(())
    }

    #[actix::test]
    async fn state_2() -> anyhow::Result<()> {
        configure_environment();
        let config = get_configuration();
        
        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();

        #[cfg(feature = "stateless")]
        cleanup_redis(conn.clone());

        DummyActor{}.start().recipient::<SendMessage>();
        let mut state = YummyState::new(config, #[cfg(feature = "stateless")] conn);
        
        state.close_session(SessionId::new());

        assert!(!state.is_session_online(SessionId::new()));
        assert!(!state.is_user_online(UserId::new()));

        Ok(())
    }
    
    #[actix::test]
    async fn room_tests() -> anyhow::Result<()> {
        configure_environment();
        let config = get_configuration();
        
        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();

        #[cfg(feature = "stateless")]
        cleanup_redis(conn.clone());

        DummyActor{}.start().recipient::<SendMessage>();
        let mut state = YummyState::new(config, #[cfg(feature = "stateless")] conn);
        
        let room_1 = RoomId::new();
        state.create_room(room_1, Some("room".to_string()), CreateRoomAccessType::Friend, None, 2, Vec::new());

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
        configure_environment();
        let config = get_configuration();

        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();

        #[cfg(feature = "stateless")]
        cleanup_redis(conn.clone());

        DummyActor{}.start().recipient::<SendMessage>();
        let mut state = YummyState::new(config, #[cfg(feature = "stateless")] conn);
    
        let room = RoomId::new();
        state.create_room(room, None, CreateRoomAccessType::Public, None, 0, Vec::new());

        for _ in 0..100_000 {
            state.join_to_room(room, UserId::new(), RoomUserType::Owner)?
        }

        Ok(())
    }
    
    #[actix::test]
    async fn get_room() -> anyhow::Result<()> {
        configure_environment();
        let config = get_configuration();

        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();

        #[cfg(feature = "stateless")]
        cleanup_redis(conn.clone());

        DummyActor{}.start().recipient::<SendMessage>();
        let mut state = YummyState::new(config, #[cfg(feature = "stateless")] conn);
    
        let room = RoomId::new();
        state.create_room(room, Some("Room 1".to_string()), CreateRoomAccessType::Private, Some("erhan".to_string()), 10, vec!["tag1".to_string(), "tag3".to_string(), "tag3".to_string()]);

        let result = state.get_room_info(room, Vec::new())?;
        assert_eq!(result.results.len(), 0);

        let result = state.get_room_info(room, vec![RoomInfoTypeVariant::RoomName])?;
        assert_eq!(result.results.len(), 1);
        assert_eq!(result.get_item(RoomInfoTypeVariant::RoomName).unwrap(), RoomInfoType::RoomName(Some("Room 1".to_string())));


        let result = state.get_room_info(room, vec![RoomInfoTypeVariant::RoomName, RoomInfoTypeVariant::Users, RoomInfoTypeVariant::MaxUser, RoomInfoTypeVariant::UserLength, RoomInfoTypeVariant::Password])?;
        assert_eq!(result.results.len(), 5);
        assert_eq!(result.get_item(RoomInfoTypeVariant::RoomName).unwrap(), RoomInfoType::RoomName(Some("Room 1".to_string())));
        assert_eq!(result.get_item(RoomInfoTypeVariant::MaxUser).unwrap(), RoomInfoType::MaxUser(10));
        assert_eq!(result.get_item(RoomInfoTypeVariant::UserLength).unwrap(), RoomInfoType::UserLength(0));
        assert_eq!(result.get_item(RoomInfoTypeVariant::Password).unwrap(), RoomInfoType::Password(Some("erhan".to_string())));
        assert!(result.get_item(RoomInfoTypeVariant::Users).is_some());

        let user_1 = UserId::new();
        let user_2 = UserId::new();
        let user_3 = UserId::new();

        state.new_session(user_1, Some("user1".to_string()));
        state.new_session(user_2, Some("user2".to_string()));
        state.new_session(user_3, Some("user3".to_string()));

        state.join_to_room(room.clone(), user_1.clone(), RoomUserType::Owner)?;
        state.join_to_room(room.clone(), user_2.clone(), RoomUserType::Owner)?;
        state.join_to_room(room.clone(), user_3.clone(), RoomUserType::Owner)?;
        
        let result = state.get_room_info(room, vec![RoomInfoTypeVariant::UserLength, RoomInfoTypeVariant::Users])?;
        assert_eq!(result.results.len(), 2);
        assert_eq!(result.get_item(RoomInfoTypeVariant::UserLength).unwrap(), RoomInfoType::UserLength(3));

        if let RoomInfoType::Users(mut users) = result.get_item(RoomInfoTypeVariant::Users).unwrap() {
            users.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
            assert_eq!(users, vec![RoomUserInformation { user_id: user_1, name: Some("user1".to_string()) }, RoomUserInformation { user_id: user_2, name: Some("user2".to_string()) }, RoomUserInformation { user_id: user_3, name: Some("user3".to_string()) }]);
        } else {
            assert!(false);
        }
        
        Ok(())
    }
}
