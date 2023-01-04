use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::{fmt::Debug, borrow::Borrow};

use actix::Message;
use serde::de::DeserializeOwned;
use serde::ser::SerializeMap;
use strum_macros::EnumDiscriminants;
use thiserror::Error;
use serde::{Serialize, Deserialize, Serializer};

#[cfg(feature = "stateless")]
use redis::Commands;

use crate::config::YummyConfig;
use crate::meta::{RoomMetaAccess, MetaType};
use crate::model::{UserId, RoomId, SessionId};
use crate::model::CreateRoomAccessType;
use crate::model::RoomUserType;
use crate::model::UserType;

#[allow(unused_macros)]
macro_rules! redis_result {
    ($query: expr) => {
        match $query {
            Ok(result) => result,
            Err(error) => {
                log::error!("Redis error: {}", error.to_string());
                Default::default()
            }
        }   
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct RoomUserInformation {
    pub user_id: Arc<UserId>,
    pub name: Option<String>,

    #[serde(rename = "type")]
    pub user_type: RoomUserType
}

#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct SendMessage {
    pub user_id: Arc<UserId>,
    pub message: String
}

impl SendMessage {
    pub fn create<T:  Borrow<T> + Debug + Serialize + DeserializeOwned>(user_id: Arc<UserId>, message: T) -> SendMessage {
        let message = serde_json::to_string(message.borrow());
        Self { user_id, message: message.unwrap() }
    }
}


#[derive(Debug, Clone, EnumDiscriminants, PartialEq, Deserialize)]
#[strum_discriminants(name(RoomInfoTypeVariant), derive(Serialize, Deserialize))]
pub enum RoomInfoType {
    RoomName(Option<String>),
    Users(Vec<RoomUserInformation>),
    MaxUser(usize),
    UserLength(usize),
    AccessType(CreateRoomAccessType),
    Tags(Vec<String>),
    Metas(HashMap<String, MetaType<RoomMetaAccess>>),
    InsertDate(i32)
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RoomInfoTypeCollection {
    pub room_id: Option<RoomId>,
    pub items: Vec<RoomInfoType>
}

impl RoomInfoTypeCollection {
    pub fn get_item(&self, query: RoomInfoTypeVariant) -> Option<RoomInfoType> {
        self.items.iter().find(|item| query == RoomInfoTypeVariant::from(item.deref())).cloned()
    }
}

impl Serialize for RoomInfoTypeCollection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut items = serializer.serialize_map(Some(self.items.len()))?;
        if let Some(room_id) = self.room_id {
            items.serialize_entry("id", &room_id)?;
        }

        for entry in self.items.iter() {
            match entry {
                RoomInfoType::RoomName(name) => items.serialize_entry("name", name),
                RoomInfoType::Users(users) => items.serialize_entry("users", users),
                RoomInfoType::MaxUser(max_user) => items.serialize_entry("max-user", max_user),
                RoomInfoType::UserLength(user_length) => items.serialize_entry("user-length", user_length),
                RoomInfoType::AccessType(access_type) => items.serialize_entry("access-type", access_type),
                RoomInfoType::Tags(tags) => items.serialize_entry("tags", tags),
                RoomInfoType::Metas(tags) => items.serialize_entry("metas", tags),
                RoomInfoType::InsertDate(insert_date) => items.serialize_entry("insert-date", insert_date),
            }?;
        }
        
        items.end()
    }
}

#[derive(Clone)]
pub struct YummyState {
    #[allow(dead_code)]
    config: Arc<YummyConfig>,

    // Fields for statefull informations
    #[cfg(not(feature = "stateless"))]
    user: Arc<parking_lot::Mutex<std::collections::HashMap<UserId, crate::model::UserState>>>,

    #[cfg(not(feature = "stateless"))]
    room: Arc<parking_lot::Mutex<std::collections::HashMap<RoomId, crate::model::RoomState>>>,
    
    #[cfg(not(feature = "stateless"))]
    session_to_user: Arc<parking_lot::Mutex<std::collections::HashMap<SessionId, UserId>>>,

    // Fields for stateless informations
    #[cfg(feature = "stateless")]
    redis: r2d2::Pool<redis::Client>
}

impl YummyState {
    pub fn new(config: Arc<YummyConfig>, #[cfg(feature = "stateless")] redis: r2d2::Pool<redis::Client>) -> Self {
        Self {
            config,

            #[cfg(not(feature = "stateless"))] user: Arc::new(parking_lot::Mutex::default()),
            #[cfg(not(feature = "stateless"))] room: Arc::new(parking_lot::Mutex::default()),
            #[cfg(not(feature = "stateless"))] session_to_user: Arc::new(parking_lot::Mutex::default()),
            
            #[cfg(feature = "stateless")] redis
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum YummyStateError {
    #[error("Room not found")]
    RoomNotFound,

    #[error("User not found")]
    UserNotFound,
    
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
    pub fn is_user_online(&mut self, user_id: &UserId) -> bool {
        match self.redis.get() {
            Ok(mut redis) => redis_result!(redis.sismember(format!("{}online-users", self.config.redis_prefix), user_id.borrow().to_string())),
            Err(_) => false
        }
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="get_user_type", skip(self))]
    pub fn get_user_type(&mut self, user_id: &UserId) -> Option<UserType> {

        match self.redis.get() {
            Ok(mut redis) => match redis_result!(redis.hget(format!("{}users:{}", self.config.redis_prefix, user_id.to_string()), "type")) {
                Some(1) => Some(UserType::User),
                Some(2) => Some(UserType::Mod),
                Some(3) => Some(UserType::Admin),
                _ => None
            },
            Err(_) => None
        }
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="get_users_room_type", skip(self))]
    pub fn get_users_room_type(&mut self, user_id: &UserId, room_id: &RoomId) -> Option<RoomUserType> {

        match self.redis.get() {
            Ok(mut redis) => match redis_result!(redis.hget(format!("{}room-users:{}", self.config.redis_prefix, room_id.to_string()), user_id.to_string())) {
                Some(1) => Some(RoomUserType::User),
                Some(2) => Some(RoomUserType::Moderator),
                Some(3) => Some(RoomUserType::Owner),
                _ => None
            },
            Err(_) => None
        }
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="set_users_room_type", skip(self))]
    pub fn set_users_room_type(&mut self, user_id: &UserId, room_id: &RoomId, user_type: RoomUserType) {
        if let Ok(mut redis) =  self.redis.get() {
            redis_result!(redis.hset::<_, _, _, i32>(format!("{}room-users:{}", self.config.redis_prefix, room_id.to_string()), user_id.to_string(), user_type as i32));
        }
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="is_session_online", skip(self))]
    pub fn is_session_online(&mut self, session_id: &SessionId) -> bool {
        match self.redis.get() {
            Ok(mut redis) => redis_result!(redis.hexists::<_, _, bool>(format!("{}session-user", self.config.redis_prefix), session_id.to_string())),
            Err(_) => false
        }
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="set_user_type", skip(self))]
    pub fn set_user_type(&self, user_id: &UserId, user_type: UserType) {
        match self.redis.get() {
            Ok(mut redis) => redis_result!(redis.hset::<_, _, _, i32>(format!("{}users:{}", self.config.redis_prefix, user_id.to_string()), "type", i32::from(user_type))),
            Err(_) => 0
        };
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="set_user_name", skip(self))]
    pub fn set_user_name(&self, user_id: &UserId, name: String) {
        match self.redis.get() {
            Ok(mut redis) => redis_result!(redis.hset::<_, _, _, i32>(format!("{}users:{}", self.config.redis_prefix, user_id.to_string()), "name", name)),
            Err(_) => 0
        };
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="new_session", skip(self))]
    pub fn new_session(&mut self, user_id: &UserId, name: Option<String>, user_type: UserType) -> SessionId {
        let session_id = SessionId::new();
        if let Ok(mut redis) = self.redis.get() {
            let user_id = user_id.to_string();
            let session_id_str = session_id.to_string();
        
            redis_result!(redis::pipe()
                .atomic()
                .cmd("SADD").arg(format!("{}online-users", self.config.redis_prefix))
                    .arg(&user_id)
                    .ignore()
                
                .cmd("HSET").arg(format!("{}session-user", self.config.redis_prefix))
                    .arg(&session_id_str).arg(&user_id)
                    .ignore()
                
                .cmd("HSET").arg(format!("{}users:{}", self.config.redis_prefix, &user_id))
                    .arg("room").arg("")
                    .arg("type").arg(i32::from(user_type))
                    .arg("name").arg(name.unwrap_or_default())
                    .arg("loc").arg(&self.config.server_name)
                    .ignore()
                .query::<()>(&mut redis));
        }
        session_id
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="close_session", skip(self))]
    pub fn close_session(&mut self, session_id: &SessionId) -> bool {
        
        match self.redis.get() {
            Ok(mut redis) => {
                let user_id = redis_result!(redis.hget::<_, _, String>(format!("{}session-user", self.config.redis_prefix), session_id.to_string()));

                let (room_id, remove_result) = redis_result!(redis::pipe()
                    .atomic()
                    .cmd("HDEL").arg(format!("{}session-user", self.config.redis_prefix))
                        .arg(session_id.borrow().to_string())
                        .ignore()
                    
                    .cmd("HGET").arg(format!("{}users:{}", self.config.redis_prefix, user_id))
                        .arg("room")

                    .cmd("DEL").arg(format!("{}users:{}", self.config.redis_prefix, user_id))
                        .ignore()
                    
                    .cmd("SREM").arg(format!("{}online-users", self.config.redis_prefix))
                        .arg(&user_id)
                    .query::<(Option<String>, i32)>(&mut redis));

                if let Some(room_id) = room_id {
                    self.disconnect_from_room(&room_id.into(), &user_id.into()).unwrap_or_default();
                }

                remove_result > 0
            },
            Err(_) => false
        }
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="get_user_room", skip(self))]
    pub fn get_user_room(&mut self, user_id: &UserId) -> Option<RoomId> {
        use std::str::FromStr;
        let result = match self.redis.get() {
            Ok(mut redis) => redis_result!(redis.hget::<_, _, String>(format!("{}users:{}", self.config.redis_prefix, user_id.to_string()), "room")),
            Err(_) => return None
        };
        
        match uuid::Uuid::from_str(&result) {
            Ok(item) => Some(RoomId::from(item)),
            Err(_) => None
        }
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="create_room", skip(self))]
    pub fn create_room(&self, room_id: &RoomId, insert_date: i32, name: Option<String>, access_type: CreateRoomAccessType, max_user: usize, tags: Vec<String>, metas: HashMap<String, MetaType<RoomMetaAccess>>) {
        if let Ok(mut redis) = self.redis.get() {
            let room_id = room_id.to_string();
            let access_type = match access_type {
                CreateRoomAccessType::Public => 1,
                CreateRoomAccessType::Private => 2,
                CreateRoomAccessType::Friend => 3,
            };

            
            let mut pipes = &mut redis::pipe();
            pipes = pipes
                .atomic()
                .cmd("SADD").arg(format!("{}rooms", self.config.redis_prefix)).arg(&room_id).ignore()
                .cmd("HSET").arg(format!("{}room:{}", self.config.redis_prefix, &room_id))
                    .arg("max-user").arg(max_user)
                    .arg("user-len").arg(0_usize)
                    .arg("name").arg(name.unwrap_or_default())
                    .arg("access").arg(access_type)
                    .arg("idate").arg(insert_date)
                    .ignore();

            if !tags.is_empty() {
                for tag in tags.iter() {
                    pipes = pipes.cmd("SADD").arg(format!("{}room-tag:{}", self.config.redis_prefix, &room_id)).arg(tag).ignore();
                    pipes = pipes.cmd("SADD").arg(format!("{}tag:{}", self.config.redis_prefix, &tag)).arg(&room_id).ignore();
                }
            }

            if !metas.is_empty() {
                let room_meta_value = format!("{}room-meta-val:{}", self.config.redis_prefix, &room_id);
                let room_meta_type = format!("{}room-meta-type:{}", self.config.redis_prefix, &room_id);
                let room_meta_per = format!("{}room-meta-acc:{}", self.config.redis_prefix, &room_id);

                for (meta, value) in metas.iter() {
                    pipes = match value {
                        MetaType::Null => pipes,
                        MetaType::Number(value, per) => {
                            pipes.cmd("HSET").arg(&room_meta_value).arg(meta).arg(value).ignore();
                            pipes.cmd("HSET").arg(&room_meta_type).arg(meta).arg(1).ignore();
                            pipes.cmd("HSET").arg(&room_meta_per).arg(meta).arg(i32::from(per.clone())).ignore()
                        },
                        MetaType::String(value, per) => {
                            pipes.cmd("HSET").arg(&room_meta_value).arg(meta).arg(value).ignore();
                            pipes.cmd("HSET").arg(&room_meta_type).arg(meta).arg(2).ignore();
                            pipes.cmd("HSET").arg(&room_meta_per).arg(meta).arg(i32::from(per.clone())).ignore()
                        },
                        MetaType::Bool(value, per) => {
                            pipes.cmd("HSET").arg(&room_meta_value).arg(meta).arg(value).ignore();
                            pipes.cmd("HSET").arg(&room_meta_type).arg(meta).arg(3).ignore();
                            pipes.cmd("HSET").arg(&room_meta_per).arg(meta).arg(i32::from(per.clone())).ignore()
                        }
                    }
                }
            }
            
            redis_result!(pipes.query::<()>(&mut redis));
        }
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="join_to_room", skip(self))]
    pub fn join_to_room(&mut self, room_id: &RoomId, user_id: &UserId, user_type: crate::model::RoomUserType) -> Result<(), YummyStateError> {

        let room_info_key = format!("{}room:{}", self.config.redis_prefix, room_id.get());
        match self.redis.get() {
            Ok(mut redis) => match redis_result!(redis.exists::<_, bool>(&room_info_key)) {
                true => {
                    let room_users_key = format!("{}room-users:{}", self.config.redis_prefix, room_id.get());
                    let user_id = user_id.to_string();

                    let room_info = redis_result!(redis::cmd("HMGET")
                        .arg(format!("{}room:{}", self.config.redis_prefix, room_id.to_string()))
                        .arg("user-len")
                        .arg("max-user")
                        .query::<Vec<usize>>(&mut redis));

                    let user_len = room_info.first().copied().unwrap_or_default();
                    let max_user = room_info.get(1).copied().unwrap_or_default();

                    // If the max_user 0 or lower than users count, add to room
                    if max_user == 0 || max_user > user_len {
                        let is_member = redis_result!(redis.hexists(&room_users_key, &user_id));
    
                        // User alread in the room
                        if is_member {
                            return Err(YummyStateError::UserAlreadInRoom);
                        }

                        redis_result!(redis::pipe()
                            .atomic()
                            .cmd("HSET").arg(format!("{}users:{}", self.config.redis_prefix, &user_id)).arg("room").arg(room_id.to_string()).ignore()
                            .cmd("HINCRBY").arg(&room_info_key).arg("user-len").arg(1).ignore()
                            .cmd("HSET").arg(room_users_key).arg(&user_id).arg(match user_type {
                                    crate::model::RoomUserType::User => 1,
                                    crate::model::RoomUserType::Moderator => 2,
                                    crate::model::RoomUserType::Owner => 3,
                                }).ignore()
                            .query::<()>(&mut redis));
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
    #[tracing::instrument(name="disconnect_from_room", skip(self))]
    pub fn disconnect_from_room(&mut self, room_id: &RoomId, user_id: &UserId) -> Result<bool, YummyStateError> {
        let room_removed: bool = match self.redis.get() {
            Ok(mut redis) => {
                let room_id = room_id.to_string();
                let room_info_key = format!("{}room:{}", self.config.redis_prefix, &room_id);
                let room_users_key = &format!("{}room-users:{}", self.config.redis_prefix, &room_id);

                let (user_len,) =  redis_result!(redis::pipe()
                    .atomic()
                    .cmd("HDEL").arg(room_users_key).arg(user_id.to_string()).ignore()
                    .cmd("HINCRBY").arg(&room_info_key).arg("user-len").arg(-1)
                    .query::<(i32,)>(&mut redis));
                    
                let no_user = user_len == 0;

                if no_user {
                    let (tags,) = redis_result!(redis::pipe()
                        .atomic()
                        .cmd("SREM").arg(format!("{}rooms", self.config.redis_prefix)).arg(&room_id).ignore()
                        .cmd("DEL").arg(room_users_key).ignore()
                        .cmd("DEL").arg(room_info_key).ignore()
                        .cmd("SMEMBERS").arg(format!("{}room-tag:{}", self.config.redis_prefix, &room_id))
                        .cmd("DEL").arg(format!("{}room-tag:{}", self.config.redis_prefix, room_id)).ignore()
                        .cmd("DEL").arg(format!("{}room-meta-val:{}", self.config.redis_prefix, room_id)).ignore()
                        .cmd("DEL").arg(format!("{}room-meta-type:{}", self.config.redis_prefix, room_id)).ignore()
                        .cmd("DEL").arg(format!("{}room-meta-acc:{}", self.config.redis_prefix, room_id)).ignore()
                        .query::<(Vec<String>,)>(&mut redis));

                    // Remove tags
                    if !tags.is_empty() {
                        let mut query = &mut redis::pipe();
                        for tag in tags.iter() {
                            query = query.cmd("SREM").arg(format!("{}tag:{}", self.config.redis_prefix, &tag)).arg(&room_id).ignore()
                        }

                        redis_result!(query.query::<()>(&mut redis));
                    }
                }

                no_user
            },
            Err(_) => false
        };
        Ok(room_removed)
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="get_users_from_room", skip(self))]
    pub fn get_users_from_room(&mut self, room_id: &RoomId) -> Result<Vec<Arc<UserId>>, YummyStateError> {
        use std::str::FromStr;
        use std::collections::HashSet;
        let users: std::collections::HashSet<String> = match self.redis.get() {
            Ok(mut redis) => match redis_result!(redis.exists::<_, bool>(&format!("{}room-users:{}", self.config.redis_prefix, room_id.get()))) {
                true => redis_result!(redis.hkeys(&format!("{}room-users:{}", self.config.redis_prefix, room_id.get()))),
                false => return Err(YummyStateError::RoomNotFound),
            },
            Err(_) => HashSet::default()
        };
        Ok(users.into_iter().map(|item| Arc::new(UserId::from(uuid::Uuid::from_str(&item[..]).unwrap_or_default()))).collect::<Vec<Arc<UserId>>>())
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="get_user_location", skip(self))]
    pub fn get_user_location(&self, user_id: Arc<UserId>) -> Option<String> {
        match self.redis.get() {
            Ok(mut redis) => match redis.hget::<_, _, String>(format!("{}users:{}", self.config.redis_prefix, user_id.as_ref().to_string()), "loc") {
                Ok(result) => Some(result),
                Err(_) => None
            },
            Err(_) => None
        }
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="get_room_info", skip(self))]
    pub fn get_room_info(&self, room_id: &RoomId, access_level: RoomMetaAccess, query: Vec<RoomInfoTypeVariant>) -> Result<RoomInfoTypeCollection, YummyStateError> {
        use std::collections::HashMap;

        use redis::FromRedisValue;

        let mut result = RoomInfoTypeCollection::default();
        if query.is_empty() {
            return Ok(result);
        }

        match self.redis.get() {
            Ok(mut redis) => {
                let room_id = room_id.to_string();
                let mut command = redis::cmd("HMGET");
                let mut request = command.arg(format!("{}room:{}", self.config.redis_prefix, &room_id));
                
                for item in query.iter() {
                    match item {
                        RoomInfoTypeVariant::RoomName => request = request.arg("name"),
                        RoomInfoTypeVariant::Users => request = request.arg("users"),
                        RoomInfoTypeVariant::MaxUser => request = request.arg("max-user"),
                        RoomInfoTypeVariant::UserLength => request = request.arg("user-len"),
                        RoomInfoTypeVariant::AccessType => request = request.arg("access"),
                        RoomInfoTypeVariant::InsertDate => request = request.arg("idate"),
                        RoomInfoTypeVariant::Tags => request = request.arg("tags"), // Dummy data, dont remove
                        RoomInfoTypeVariant::Metas => request = request.arg("metas"), // Dummy data, dont remove
                    };
                }

                let room_infos = redis_result!(request.query::<Vec<redis::Value>>(&mut redis));

                for (query, room_info) in query.into_iter().zip(room_infos.into_iter()) {
                    match query {
                        RoomInfoTypeVariant::RoomName => {
                            let room_name: String = FromRedisValue::from_redis_value(&room_info).unwrap_or_default();
                            result.items.push(RoomInfoType::RoomName(if room_name.is_empty() { None } else { Some(room_name) }));
                        },
                        RoomInfoTypeVariant::Users => {

                            // This request is slow compare to other. We should change it to lua script to increase performance
                            let mut user_infos = Vec::new();
                            let users = redis_result!(redis.hgetall::<_, HashMap<String, i32>>(format!("{}room-users:{}", self.config.redis_prefix, &room_id)));
                            for (user_id, user_type) in users.iter() {
                                let name = redis_result!(redis.hget::<_, _, String>(format!("{}users:{}", self.config.redis_prefix, user_id), "name"));
                                user_infos.push(RoomUserInformation {
                                    name: if name.is_empty() { None } else { Some(name) },
                                    user_id: Arc::new(UserId::from(uuid::Uuid::parse_str(user_id).unwrap_or_default())),
                                    user_type: match user_type {
                                        1 => RoomUserType::User,
                                        2 => RoomUserType::Moderator,
                                        3 => RoomUserType::Owner,
                                        _ => RoomUserType::User
                                    }
                                })
                            }
                            result.items.push(RoomInfoType::Users(user_infos));
                        },
                        RoomInfoTypeVariant::AccessType => result.items.push(RoomInfoType::AccessType(match FromRedisValue::from_redis_value(&room_info).unwrap_or_default() {
                            1 => CreateRoomAccessType::Public,
                            2 => CreateRoomAccessType::Private,
                            3 => CreateRoomAccessType::Friend,
                            _ => CreateRoomAccessType::Public
                        })),
                        RoomInfoTypeVariant::InsertDate => result.items.push(RoomInfoType::InsertDate(FromRedisValue::from_redis_value(&room_info).unwrap_or_default())),
                        RoomInfoTypeVariant::MaxUser => result.items.push(RoomInfoType::MaxUser(FromRedisValue::from_redis_value(&room_info).unwrap_or_default())),
                        RoomInfoTypeVariant::UserLength => result.items.push(RoomInfoType::UserLength(FromRedisValue::from_redis_value(&room_info).unwrap_or_default())),
                        RoomInfoTypeVariant::Tags => {
                            let tags = redis_result!(redis.smembers::<_, Vec<String>>(format!("{}room-tag:{}", self.config.redis_prefix, &room_id)));
                            result.items.push(RoomInfoType::Tags(tags));
                        },
                        RoomInfoTypeVariant::Metas => {
                            let access_level = i32::from(access_level.clone());

                            let access_map = redis_result!(redis.hgetall::<_, HashMap<String, i32>>(format!("{}room-meta-acc:{}", self.config.redis_prefix, &room_id)));
                            let mut keys = Vec::new();
                            let mut access = Vec::new();

                            for (key, value) in access_map.into_iter() {
                                if value <= access_level {
                                    keys.push(key);
                                    access.push(value);
                                }
                            }
                            
                            let mut pipe = redis::pipe();

                            {
                                let command = pipe.cmd("HMGET");
                                let mut query = command.arg(format!("{}room-meta-val:{}", self.config.redis_prefix, &room_id));

                                for key in keys.iter() {
                                    query = query.arg(key);
                                }
                            }

                            {
                                let command = pipe.cmd("HMGET");
                                let mut query = command.arg(format!("{}room-meta-type:{}", self.config.redis_prefix, &room_id));

                                for key in keys.iter() {
                                    query = query.arg(key);
                                }
                            }


                            let (values, types) = redis_result!(pipe.query::<(Vec<redis::Value>, Vec<i32>)>(&mut redis));
                            let mut metas = HashMap::new();
                            
                            for (((key, type_info), value), access) in keys.into_iter().zip(types.into_iter()).zip(values.into_iter()).zip(access.into_iter()) {
                                let value = match type_info {
                                    1 => MetaType::Number(FromRedisValue::from_redis_value(&value).unwrap_or_default(), RoomMetaAccess::from(access)),
                                    2 => MetaType::String(FromRedisValue::from_redis_value(&value).unwrap_or_default(), RoomMetaAccess::from(access)),
                                    3 => MetaType::Bool(FromRedisValue::from_redis_value(&value).unwrap_or_default(), RoomMetaAccess::from(access)),
                                    _ => MetaType::Number(FromRedisValue::from_redis_value(&value).unwrap_or_default(), RoomMetaAccess::from(access)),
                                };

                                metas.insert(key, value);
                            }

                            result.items.push(RoomInfoType::Metas(metas));
                        },
                    };
                }

                Ok(result)
            },
            Err(_) => Err(YummyStateError::CacheCouldNotReaded)
        }
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="set_room_info", skip(self))]
    pub fn set_room_info(&self, room_id: &RoomId, query: Vec<RoomInfoType>) {
        if query.is_empty() {
            return;
        }

        match self.redis.get() {
            Ok(mut redis) => {
                let mut command = &mut redis::pipe();
                let room_id = room_id.to_string();

                for item in query.into_iter() {
                    match item {
                        RoomInfoType::RoomName(name) => command = command.cmd("HSET").arg(format!("{}room:{}", self.config.redis_prefix, &room_id)).arg("name").arg(name.unwrap_or_default()).ignore(),
                        RoomInfoType::Users(_) => (),
                        RoomInfoType::MaxUser(max_user) => command = command.cmd("HSET").arg(format!("{}room:{}", self.config.redis_prefix, &room_id)).arg("max-user").arg(max_user).ignore(),
                        RoomInfoType::UserLength(_) => (),
                        RoomInfoType::AccessType(access_type) => command = command.cmd("HSET").arg(format!("{}room:{}", self.config.redis_prefix, &room_id)).arg("access").arg(i32::from(access_type)).ignore(),
                        RoomInfoType::Tags(tags) => {
                            
                            // Remove old tags
                            let saved_tags = redis_result!(redis.smembers::<_, Vec<String>>(format!("{}room-tag:{}", self.config.redis_prefix, room_id.to_string())));
                            for tag in saved_tags.iter() {
                                command = command.cmd("SREM").arg(format!("{}tag:{}", self.config.redis_prefix, &tag)).arg(&room_id).ignore()
                            }

                            command = command.cmd("DEL").arg(format!("{}room-tag:{}", self.config.redis_prefix, room_id)).ignore();
                            
                            if !tags.is_empty() {
                                // Add to 
                                command = command.cmd("SADD").arg(format!("{}room-tag:{}", self.config.redis_prefix, &room_id));
                                for tag in tags.iter() {
                                    command = command.arg(tag);
                                }
                                command = command.ignore();
                                
                                for tag in tags.iter() {
                                    command = command.cmd("SADD").arg(format!("{}tag:{}", self.config.redis_prefix, &tag)).arg(&room_id).ignore();
                                }
                            }
                        },
                        RoomInfoType::InsertDate(_) => (),
                        RoomInfoType::Metas(_) => (),
                    };
                }

                redis_result!(command.query::<()>(&mut redis));
            },
            Err(_) => ()
        }
    }

    #[cfg(feature = "stateless")]
    #[tracing::instrument(name="get_rooms", skip(self))]
    pub fn get_rooms(&self, tag: Option<String>, query: Vec<RoomInfoTypeVariant>) -> Result<Vec<RoomInfoTypeCollection>, YummyStateError> {
        use std::collections::HashMap;

        use redis::FromRedisValue;
        
        match self.redis.get() {
            Ok(mut redis) => {
                let mut results = Vec::new();
                
                let rooms = match tag {
                    Some(tag) => redis_result!(redis.smembers::<_, Vec<String>>(format!("{}tag:{}", self.config.redis_prefix, &tag))),
                    None => redis_result!(redis.smembers::<_, Vec<String>>(format!("{}rooms", self.config.redis_prefix)))
                };

                let mut command = &mut redis::pipe();
                for room_id in rooms.iter() {
                    command = command.cmd("HMGET").arg(format!("{}room:{}", self.config.redis_prefix, &room_id));
                    
                    for item in query.iter() {
                        match item {
                            RoomInfoTypeVariant::RoomName => command = command.arg("name"),
                            RoomInfoTypeVariant::Users => command = command.arg("users"),
                            RoomInfoTypeVariant::MaxUser => command = command.arg("max-user"),
                            RoomInfoTypeVariant::UserLength => command = command.arg("user-len"),
                            RoomInfoTypeVariant::AccessType => command = command.arg("access"),
                            RoomInfoTypeVariant::InsertDate => command = command.arg("idate"),
                            RoomInfoTypeVariant::Tags => command = command.arg("tags"), // Dummy data, dont remove
                            RoomInfoTypeVariant::Metas => command = command.arg("metas"), // Dummy data, dont remove
                        };
                    }
                }

                let room_results = redis_result!(command.query::<Vec<redis::Value>>(&mut redis));

                for (room_id, room_result) in rooms.into_iter().zip(room_results.into_iter()) {
                    let room_id = RoomId::from(uuid::Uuid::parse_str(&room_id).unwrap_or_default());

                    let mut room_info = RoomInfoTypeCollection {
                        room_id: Some(room_id),
                        .. Default::default()
                    };

                    // Get all sub results
                    let room_result: Vec<redis::Value> = FromRedisValue::from_redis_value(&room_result).unwrap_or_default();

                    for (index, item) in query.iter().enumerate() {
                        let redis_value = room_result.get(index).unwrap_or(&redis::Value::Nil);

                        match item {
                            RoomInfoTypeVariant::RoomName => {
                                let room_name: String = FromRedisValue::from_redis_value(redis_value).unwrap_or_default();
                                room_info.items.push(RoomInfoType::RoomName(if room_name.is_empty() { None } else { Some(room_name) }));
                            },
                            RoomInfoTypeVariant::Users => {
    
                                // This request is slow compare to other. We should change it to lua script to increase performance
                                let mut user_infos = Vec::new();
                                let users = redis_result!(redis.hgetall::<_, HashMap<String, i32>>(format!("{}room-users:{}", self.config.redis_prefix, room_id.get())));
                                for (user_id, user_type) in users.iter() {
                                    let name = redis_result!(redis.hget::<_, _, String>(format!("{}users:{}", self.config.redis_prefix, user_id), "name"));
                                    user_infos.push(RoomUserInformation {
                                        name: if name.is_empty() { None } else { Some(name) },
                                        user_id: Arc::new(UserId::from(uuid::Uuid::parse_str(user_id).unwrap_or_default())),
                                        user_type: match user_type {
                                            1 => RoomUserType::User,
                                            2 => RoomUserType::Owner,
                                            3 => RoomUserType::Moderator,
                                            _ => RoomUserType::User
                                        }
                                    })
                                }
                                room_info.items.push(RoomInfoType::Users(user_infos));
                            },
                            RoomInfoTypeVariant::AccessType => room_info.items.push(RoomInfoType::AccessType(match FromRedisValue::from_redis_value(redis_value).unwrap_or_default() {
                                1 => CreateRoomAccessType::Public,
                                2 => CreateRoomAccessType::Private,
                                3 => CreateRoomAccessType::Friend,
                                _ => CreateRoomAccessType::Public
                            })),
                            RoomInfoTypeVariant::InsertDate => room_info.items.push(RoomInfoType::InsertDate(FromRedisValue::from_redis_value(redis_value).unwrap_or_default())),
                            RoomInfoTypeVariant::MaxUser => room_info.items.push(RoomInfoType::MaxUser(FromRedisValue::from_redis_value(redis_value).unwrap_or_default())),
                            RoomInfoTypeVariant::UserLength => room_info.items.push(RoomInfoType::UserLength(FromRedisValue::from_redis_value(redis_value).unwrap_or_default())),
                            RoomInfoTypeVariant::Tags => {
                                let tags = redis_result!(redis.smembers::<_, Vec<String>>(format!("{}room-tag:{}", self.config.redis_prefix, room_id.to_string())));
                                room_info.items.push(RoomInfoType::Tags(tags));
                            },
                            RoomInfoTypeVariant::Metas => {
                            }
                        }
                    }
        
                    results.push(room_info);
                }

                Ok(results)
            },
            Err(_) => Err(YummyStateError::CacheCouldNotReaded)
        }
    }

    /* STATEFULL functions */
    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="is_user_online", skip(self))]
    pub fn is_user_online(&self, user_id: &UserId) -> bool {
        self.user.lock().contains_key(user_id)
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="get_user_type", skip(self))]
    pub fn get_user_type(&mut self, user_id: &UserId) -> Option<UserType> {
        self.user.lock().get(user_id).map(|user| user.user_type)
    }
    
    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="is_user_online", skip(self))]
    pub fn get_users_room_type(&mut self, user_id: &UserId, room_id: &RoomId) -> Option<RoomUserType> {
        match self.room.lock().get(room_id) {
            Some(room) => room.users.lock().get(user_id).cloned(),
            None => None
        }
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="set_users_room_type", skip(self))]
    pub fn set_users_room_type(&mut self, user_id: &UserId, room_id: &RoomId, user_type: RoomUserType) {
        if let Some(room) = self.room.lock().get_mut(room_id.borrow()) {
            match room.users.lock().get_mut(user_id) {
                Some(user) => *user = user_type,
                None => ()
            };
        }
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="is_session_online", skip(self))]
    pub fn is_session_online<T: Borrow<SessionId> + std::fmt::Debug>(&self, session_id: T) -> bool {
        self.session_to_user.lock().contains_key(session_id.borrow())
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="new_session", skip(self))]
    pub fn new_session(&self, user_id: &UserId, name: Option<String>, user_type: UserType) -> SessionId {
        use std::cell::Cell;

        let session_id = SessionId::new();
        self.session_to_user.lock().insert(session_id.clone(), user_id.clone()); // todo: discart cloning
        self.user.lock().insert(user_id.clone(), crate::model::UserState { user_id: user_id.clone(), name, session: session_id.clone(), user_type, room: Cell::new(None) }); // todo: discart cloning
        session_id
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="set_user_type", skip(self))]
    pub fn set_user_type(&self, user_id: &UserId, user_type: UserType) {
        if let Some(user) = self.user.lock().get_mut(user_id) {
            user.user_type = user_type
        }
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="set_user_name", skip(self))]
    pub fn set_user_name(&self, user_id: &UserId, name: String) {
        if let Some(user) = self.user.lock().get_mut(user_id) {
            user.name = Some(name)
        }
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="close_session", skip(self))]
    pub fn close_session(&self, session_id: &SessionId) -> bool {
        let removed = self.session_to_user.lock().remove(session_id);

        match removed {
            Some(removed) => self.user.lock().remove(&removed).map(|_| true).unwrap_or_default(),
            None => false
        }
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="get_user_room", skip(self))]
    pub fn get_user_room(&self, user_id: &UserId) -> Option<RoomId> {
        self.user.lock().get(user_id.borrow()).and_then(|user| user.room.get())
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="create_room", skip(self))]
    pub fn create_room(&self, room_id: &RoomId, insert_date: i32, name: Option<String>, access_type: CreateRoomAccessType, max_user: usize, tags: Vec<String>, metas: HashMap<String, MetaType<RoomMetaAccess>>) {
        use std::collections::HashMap;

        self.room.lock().insert(*room_id, crate::model::RoomState { max_user, room_id: *room_id, insert_date, users: parking_lot::Mutex::new(HashMap::new()), tags, name, access_type, metas });
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="join_to_room", skip(self))]
    pub fn join_to_room(&self, room_id: &RoomId, user_id: &UserId, user_type: crate::model::RoomUserType) -> Result<(), YummyStateError> {
        match self.room.lock().get_mut(room_id.borrow()) {
            Some(room) => {
                let mut users = room.users.lock();
                let users_len = users.len();

                // If the max_user 0 or lower than users count, add to room
                if room.max_user == 0 || room.max_user > users_len {

                    // User alread in the room
                    if users.insert(user_id.clone(), user_type).is_some() {
                        return Err(YummyStateError::UserAlreadInRoom);
                    }
                    
                    match self.user.lock().get(user_id) {
                        Some(user) => user.room.set(Some(room_id.clone())),
                        None => return Err(YummyStateError::UserNotFound)
                    };
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
    pub fn disconnect_from_room(&self, room_id: &RoomId, user_id: &UserId) -> Result<bool, YummyStateError> {
        let mut rooms = self.room.lock();
        let room_removed = match rooms.get_mut(room_id.borrow()) {
            Some(room) => {
                let mut users = room.users.lock();

                let user_removed = users.remove(user_id);

                match user_removed.is_some() {
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
    pub fn get_users_from_room(&self, room_id: &RoomId) -> Result<Vec<Arc<UserId>>, YummyStateError> {
        match self.room.lock().get_mut(room_id) {
            Some(room) => Ok(room.users.lock().keys().map(|item| Arc::new(item.clone())).collect::<Vec<_>>()), // todo: discart cloning
            None => Err(YummyStateError::RoomNotFound)
        }
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="get_user_location", skip(self))]
    pub fn get_user_location(&self, user_id: &UserId) -> Option<String> {
        None
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="get_room_info", skip(self))]
    pub fn get_room_info(&self, room_id: &RoomId, access_level: RoomMetaAccess, query: Vec<RoomInfoTypeVariant>) -> Result<RoomInfoTypeCollection, YummyStateError> {
        let mut result = RoomInfoTypeCollection::default();
        match self.room.lock().get(room_id) {
            Some(room) => {
                for item in query.iter() {
                    let item = match item {
                        RoomInfoTypeVariant::InsertDate => RoomInfoType::InsertDate(room.insert_date),
                        RoomInfoTypeVariant::MaxUser => RoomInfoType::MaxUser(room.max_user),
                        RoomInfoTypeVariant::RoomName => RoomInfoType::RoomName(room.name.clone()),
                        RoomInfoTypeVariant::UserLength => RoomInfoType::UserLength(room.users.lock().len()),
                        RoomInfoTypeVariant::AccessType => RoomInfoType::AccessType(room.access_type.clone()),
                        RoomInfoTypeVariant::Users => {
                            let  mut users = Vec::new();
                            for (user_id, user_type) in room.users.lock().iter() {
                                let name = match self.user.lock().get(user_id) {
                                    Some(user) => user.name.clone(),
                                    None => None
                                };
                                users.push(RoomUserInformation {
                                    user_id: Arc::new(user_id.clone()), // todo: discart cloning
                                    name,
                                    user_type: user_type.clone()
                                });
                            }

                            RoomInfoType::Users(users)
                        },
                        RoomInfoTypeVariant::Tags => RoomInfoType::Tags(room.tags.clone()),
                        RoomInfoTypeVariant::Metas => {
                            let metas: HashMap<String, MetaType<RoomMetaAccess>> = room.metas
                                .iter()
                                .filter(|(_, value)| value.get_access_level() <= access_level)
                                .map(|(key, value)| (key.clone(), value.clone()))
                                .collect();
                            RoomInfoType::Metas(metas)
                        }
                    };
        
                    result.items.push(item);
                }

                Ok(result)
            },
            None => Err(YummyStateError::RoomNotFound)
        }
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="set_room_info", skip(self))]
    pub fn set_room_info(&self, room_id: &RoomId, query: Vec<RoomInfoType>) {
        match self.room.lock().get_mut(room_id) {
            Some(room) => {
                for item in query.into_iter() {
                    match item {
                        RoomInfoType::RoomName(name) => room.name = name,
                        RoomInfoType::Users(_) => (),
                        RoomInfoType::MaxUser(max_user) => room.max_user = max_user,
                        RoomInfoType::UserLength(_) => (),
                        RoomInfoType::AccessType(access_type) => room.access_type = access_type,
                        RoomInfoType::Tags(tags) => room.tags = tags,
                        RoomInfoType::Metas(metas) => room.metas = metas,
                        RoomInfoType::InsertDate(_) => (),
                    };
                }
            },
            None => ()
        }
    }

    #[cfg(not(feature = "stateless"))]
    #[tracing::instrument(name="get_rooms", skip(self))]
    pub fn get_rooms(&self, tag: Option<String>, query: Vec<RoomInfoTypeVariant>) -> Result<Vec<RoomInfoTypeCollection>, YummyStateError> {
        let mut result = Vec::default();
        let rooms = self.room.lock();
        let rooms = match tag {
            Some(tag) => rooms.iter().filter(|item| item.1.tags.contains(&tag)).collect::<Vec<_>>(),
            None => rooms.iter().collect::<Vec<_>>()
        };

        for (room_id, room_state) in rooms.into_iter() {
            let mut room_info = RoomInfoTypeCollection {
                room_id: Some(*room_id),
                .. Default::default()
            };

            for item in query.iter() {
                match item {
                    RoomInfoTypeVariant::InsertDate => room_info.items.push(RoomInfoType::InsertDate(room_state.insert_date)),
                    RoomInfoTypeVariant::MaxUser => room_info.items.push(RoomInfoType::MaxUser(room_state.max_user)),
                    RoomInfoTypeVariant::RoomName => room_info.items.push(RoomInfoType::RoomName(room_state.name.clone())),
                    RoomInfoTypeVariant::UserLength => room_info.items.push(RoomInfoType::UserLength(room_state.users.lock().len())),
                    RoomInfoTypeVariant::AccessType => room_info.items.push(RoomInfoType::AccessType(room_state.access_type.clone())),
                    RoomInfoTypeVariant::Users => {
                        let  mut users = Vec::new();
                        for (user_id, user_type) in room_state.users.lock().iter() {
                            let name = match self.user.lock().get(user_id) {
                                Some(user) => user.name.clone(),
                                None => None
                            };
                            users.push(RoomUserInformation {
                                user_id: Arc::new(user_id.clone()),
                                name,
                                user_type: user_type.clone()
                            });
                        }
                        room_info.items.push(RoomInfoType::Users(users))
                    },
                    RoomInfoTypeVariant::Tags => room_info.items.push(RoomInfoType::Tags(room_state.tags.clone())),
                    RoomInfoTypeVariant::Metas => room_info.items.push(RoomInfoType::Metas(room_state.metas.clone()))
                };
            }

            result.push(room_info);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::configure_environment;
    use crate::{model::*, config::get_configuration};

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


        DummyActor{}.start().recipient::<SendMessage>();
        let mut state = YummyState::new(config, #[cfg(feature = "stateless")] conn);
        let user_id = UserId::new();
        let session_id = state.new_session(&user_id, None, UserType::Mod);
        assert_eq!(state.get_user_type(&user_id), Some(UserType::Mod));

        assert!(state.is_session_online(&session_id));
        assert!(state.is_user_online(&user_id));

        state.close_session(&session_id);

        assert!(!state.is_session_online(&session_id));
        assert!(!state.is_user_online(&user_id));

        Ok(())
    }

    #[actix::test]
    async fn state_2() -> anyhow::Result<()> {
        configure_environment();
        let config = get_configuration();
        
        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();


        DummyActor{}.start().recipient::<SendMessage>();
        let mut state = YummyState::new(config, #[cfg(feature = "stateless")] conn);
        
        state.close_session(&SessionId::new());

        assert!(!state.is_session_online(&SessionId::new()));
        assert!(!state.is_user_online(&UserId::new()));

        Ok(())
    }
    
    #[actix::test]
    async fn room_tests() -> anyhow::Result<()> {
        configure_environment();
        let config = get_configuration();
        
        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();


        DummyActor{}.start().recipient::<SendMessage>();
        let mut state = YummyState::new(config, #[cfg(feature = "stateless")] conn);
        
        let room_1 = RoomId::new();
        state.create_room(&room_1, 1234, Some("room".to_string()), CreateRoomAccessType::Friend, 2, vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()], HashMap::from([
            ("gender".to_string(), MetaType::String("Male".to_string(), RoomMetaAccess::User)),
            ("location".to_string(), MetaType::String("Copenhagen".to_string(), RoomMetaAccess::User)),
            ("postcode".to_string(), MetaType::Number(1000.0, RoomMetaAccess::Moderator)),
            ("score".to_string(), MetaType::Number(15.3, RoomMetaAccess::Anonymous)),
            ("temp_admin".to_string(), MetaType::Bool(true, RoomMetaAccess::Admin)),
        ]));

        let user_1 = UserId::new();
        let user_2 = UserId::new();
        let user_3 = UserId::new();

        state.new_session(&user_1, None, UserType::User);
        state.new_session(&user_2, None, UserType::User);
        state.new_session(&user_3, None, UserType::User);
        
        state.join_to_room(&room_1, &user_1, RoomUserType::Owner)?;
        assert_eq!(state.get_users_room_type(&user_1, &room_1).unwrap(), RoomUserType::Owner);

        assert_eq!(state.join_to_room(&room_1, &user_1, RoomUserType::Owner).err().unwrap(), YummyStateError::UserAlreadInRoom);

        state.join_to_room(&room_1, &user_2, RoomUserType::User)?;
        assert_eq!(state.get_users_room_type(&user_2, &room_1).unwrap(), RoomUserType::User);

        assert_eq!(state.join_to_room(&room_1, &user_3, RoomUserType::Owner).err().unwrap(), YummyStateError::RoomHasMaxUsers);
        assert_eq!(state.join_to_room(&room_1, &user_2, RoomUserType::Owner).err().unwrap(), YummyStateError::RoomHasMaxUsers);

        assert_eq!(state.join_to_room(&RoomId::new(), &UserId::new(), RoomUserType::Owner).err().unwrap(), YummyStateError::RoomNotFound);
        assert_eq!(state.get_users_from_room(&room_1)?.len(), 2);

        assert_eq!(state.disconnect_from_room(&room_1, &user_1)?, false);
        assert_eq!(state.get_users_from_room(&room_1)?.len(), 1);

        assert_eq!(state.disconnect_from_room(&room_1, &&user_2)?, true);
        assert!(state.get_users_from_room(&room_1).is_err());

        Ok(())
    }
    
    #[actix::test]
    async fn room_unlimited_users_tests() -> anyhow::Result<()> {
        configure_environment();
        let config = get_configuration();

        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();


        DummyActor{}.start().recipient::<SendMessage>();
        let mut state = YummyState::new(config, #[cfg(feature = "stateless")] conn);
    
        let room = RoomId::new();
        state.create_room(&room, 1234, None, CreateRoomAccessType::Public, 0, Vec::new(), HashMap::default());

        for _ in 0..100_000 {
            let user_id = UserId::new();
            state.new_session(&user_id, None, UserType::User);
            state.join_to_room(&room, &user_id, RoomUserType::Owner)?
        }

        Ok(())
    }
    
    #[actix::test]
    async fn get_room() -> anyhow::Result<()> {
        configure_environment();
        let config = get_configuration();

        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();


        DummyActor{}.start().recipient::<SendMessage>();
        let mut state = YummyState::new(config, #[cfg(feature = "stateless")] conn);
    
        let room = RoomId::new();
        state.create_room(&room, 1234, Some("Room 1".to_string()), CreateRoomAccessType::Private, 10, vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()], HashMap::default());

        let result = state.get_room_info(&room, RoomMetaAccess::Admin, Vec::new())?;
        assert_eq!(result.items.len(), 0);

        let result = state.get_room_info(&room, RoomMetaAccess::Admin, vec![RoomInfoTypeVariant::RoomName])?;
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.get_item(RoomInfoTypeVariant::RoomName).unwrap(), RoomInfoType::RoomName(Some("Room 1".to_string())));

        state.set_room_info(&room, vec![RoomInfoType::RoomName(Some("New room".to_string()))]);

        let result = state.get_room_info(&room, RoomMetaAccess::Admin, vec![RoomInfoTypeVariant::RoomName])?;
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.get_item(RoomInfoTypeVariant::RoomName).unwrap(), RoomInfoType::RoomName(Some("New room".to_string())));

        let result = state.get_room_info(&room, RoomMetaAccess::Admin, vec![RoomInfoTypeVariant::Tags, RoomInfoTypeVariant::InsertDate, RoomInfoTypeVariant::RoomName, RoomInfoTypeVariant::AccessType, RoomInfoTypeVariant::Users, RoomInfoTypeVariant::MaxUser, RoomInfoTypeVariant::UserLength])?;
        assert_eq!(result.items.len(), 7);
        assert_eq!(result.get_item(RoomInfoTypeVariant::RoomName).unwrap(), RoomInfoType::RoomName(Some("New room".to_string())));
        assert_eq!(result.get_item(RoomInfoTypeVariant::MaxUser).unwrap(), RoomInfoType::MaxUser(10));
        assert_eq!(result.get_item(RoomInfoTypeVariant::UserLength).unwrap(), RoomInfoType::UserLength(0));
        assert_eq!(result.get_item(RoomInfoTypeVariant::AccessType).unwrap(), RoomInfoType::AccessType(CreateRoomAccessType::Private));
        assert!(result.get_item(RoomInfoTypeVariant::Users).is_some());
        assert!(result.get_item(RoomInfoTypeVariant::Tags).is_some());
        assert!(result.get_item(RoomInfoTypeVariant::InsertDate).is_some());

        // Tag update test
        let tags = result.get_item(RoomInfoTypeVariant::Tags);
        if let Some(RoomInfoType::Tags(mut tags)) = tags {
            tags.sort_by(|a, b| a.partial_cmp(b).unwrap());
            assert_eq!(tags, vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()]);
        } else {
            assert!(false, "tags not found")
        }

        state.set_room_info(&room, vec![RoomInfoType::Tags(vec!["yummy1".to_string(), "yummy2".to_string(), "yummy3".to_string()])]);
        let result = state.get_room_info(&room, RoomMetaAccess::Admin, vec![RoomInfoTypeVariant::Tags])?;
        
        let tags = result.get_item(RoomInfoTypeVariant::Tags);
        if let Some(RoomInfoType::Tags(mut tags)) = tags {
            tags.sort_by(|a, b| a.partial_cmp(b).unwrap());
            assert_eq!(tags, vec!["yummy1".to_string(), "yummy2".to_string(), "yummy3".to_string()]);
        } else {
            assert!(false, "tags not found")
        }

        state.set_room_info(&room, vec![RoomInfoType::Tags(Vec::new())]);
        let result = state.get_room_info(&room, RoomMetaAccess::Admin, vec![RoomInfoTypeVariant::Tags])?;
        
        let tags = result.get_item(RoomInfoTypeVariant::Tags);
        if let Some(RoomInfoType::Tags(tags)) = tags {
            assert_eq!(tags, Vec::<String>::new());
        } else {
            assert!(false, "tags not found")
        }

        let user_1 = UserId::new();
        let user_2 = UserId::new();
        let user_3 = UserId::new();

        state.new_session(&user_1, Some("user1".to_string()), UserType::User);
        assert_eq!(state.get_user_type(&user_1), Some(UserType::User));

        state.new_session(&user_2, Some("user2".to_string()), UserType::Mod);
        assert_eq!(state.get_user_type(&user_2), Some(UserType::Mod));

        state.new_session(&user_3, Some("user3".to_string()), UserType::Admin);
        assert_eq!(state.get_user_type(&user_3), Some(UserType::Admin));

        state.join_to_room(&room, &user_1, RoomUserType::Owner)?;
        state.join_to_room(&room, &user_2, RoomUserType::Owner)?;
        state.join_to_room(&room, &user_3, RoomUserType::Owner)?;
        
        let result = state.get_room_info(&room, RoomMetaAccess::Admin, vec![RoomInfoTypeVariant::UserLength, RoomInfoTypeVariant::Users])?;
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.get_item(RoomInfoTypeVariant::UserLength).unwrap(), RoomInfoType::UserLength(3));

        if let RoomInfoType::Users(mut users) = result.get_item(RoomInfoTypeVariant::Users).unwrap() {
            users.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
            assert_eq!(users, vec![RoomUserInformation { user_id: Arc::new(user_1.clone()), name: Some("user1".to_string()), user_type: RoomUserType::Owner }, RoomUserInformation { user_id: Arc::new(user_2.clone()), name: Some("user2".to_string()), user_type: RoomUserType::Owner }, RoomUserInformation { user_id: Arc::new(user_3.clone()), name: Some("user3".to_string()), user_type: RoomUserType::Owner }]);
        } else {
            assert!(false);
        }

        // Change user permission
        state.set_users_room_type(&user_1, &room, RoomUserType::User);
        
        let result = state.get_room_info(&room, RoomMetaAccess::Admin, vec![RoomInfoTypeVariant::Users])?;
        assert_eq!(result.items.len(), 1);

        if let RoomInfoType::Users(mut users) = result.get_item(RoomInfoTypeVariant::Users).unwrap() {
            users.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
            assert_eq!(users, vec![RoomUserInformation { user_id: Arc::new(user_1), name: Some("user1".to_string()), user_type: RoomUserType::User }, RoomUserInformation { user_id: Arc::new(user_2), name: Some("user2".to_string()), user_type: RoomUserType::Owner }, RoomUserInformation { user_id: Arc::new(user_3), name: Some("user3".to_string()), user_type: RoomUserType::Owner }]);
        } else {
            assert!(false);
        }
        
        Ok(())
    }
    
    macro_rules! meta_validation {
        ($state: expr, $room_id: expr, $access: expr, $len: expr, $map: expr) => {
            let metas = $state.get_room_info(&$room_id, $access, vec![RoomInfoTypeVariant::Metas])?;
            let item = metas.get_item(RoomInfoTypeVariant::Metas);
            assert!(item.is_some());

            println!("{:?}", metas);
    
            if let Some(RoomInfoType::Metas(metas)) = item {
                assert_eq!(metas.len(), $len);
                assert_eq!(metas, $map);
            } else {
                assert!(false, "Metas not found");
            }
        }
    }

    #[actix::test]
    async fn room_meta_read_test() -> anyhow::Result<()> {
        configure_environment();
        let config = get_configuration();
        
        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();

        DummyActor{}.start().recipient::<SendMessage>();
        let mut state = YummyState::new(config, #[cfg(feature = "stateless")] conn);
        
        let room_id = RoomId::new();
        state.create_room(&room_id, 1234, Some("room".to_string()), CreateRoomAccessType::Friend, 2, vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()], HashMap::from([
            ("gender".to_string(), MetaType::String("Male".to_string(), RoomMetaAccess::User)),
            ("location".to_string(), MetaType::String("Copenhagen".to_string(), RoomMetaAccess::User)),
            ("postcode".to_string(), MetaType::Number(1000.0, RoomMetaAccess::Moderator)),
            ("score".to_string(), MetaType::Number(15.3, RoomMetaAccess::Anonymous)),
            ("temp_admin".to_string(), MetaType::Bool(true, RoomMetaAccess::Admin)),
        ]));

        meta_validation!(state, room_id, RoomMetaAccess::Anonymous, 1, HashMap::from([
            ("score".to_string(), MetaType::Number(15.3, RoomMetaAccess::Anonymous))
        ]));

        meta_validation!(state, room_id, RoomMetaAccess::User, 3, HashMap::from([
            ("gender".to_string(), MetaType::String("Male".to_string(), RoomMetaAccess::User)),
            ("location".to_string(), MetaType::String("Copenhagen".to_string(), RoomMetaAccess::User)),
            ("score".to_string(), MetaType::Number(15.3, RoomMetaAccess::Anonymous)),
        ]));

        meta_validation!(state, room_id, RoomMetaAccess::Moderator, 4, HashMap::from([
            ("gender".to_string(), MetaType::String("Male".to_string(), RoomMetaAccess::User)),
            ("location".to_string(), MetaType::String("Copenhagen".to_string(), RoomMetaAccess::User)),
            ("postcode".to_string(), MetaType::Number(1000.0, RoomMetaAccess::Moderator)),
            ("score".to_string(), MetaType::Number(15.3, RoomMetaAccess::Anonymous)),
        ]));

        meta_validation!(state, room_id, RoomMetaAccess::Admin, 5, HashMap::from([
            ("gender".to_string(), MetaType::String("Male".to_string(), RoomMetaAccess::User)),
            ("location".to_string(), MetaType::String("Copenhagen".to_string(), RoomMetaAccess::User)),
            ("postcode".to_string(), MetaType::Number(1000.0, RoomMetaAccess::Moderator)),
            ("score".to_string(), MetaType::Number(15.3, RoomMetaAccess::Anonymous)),
            ("temp_admin".to_string(), MetaType::Bool(true, RoomMetaAccess::Admin)),
        ]));

        Ok(())
    }
}
