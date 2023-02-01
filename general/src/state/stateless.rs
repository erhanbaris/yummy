use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::borrow::Borrow;

use redis::Commands;
use super::*;

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
                //panic!("{}", error);
            }
        }   
    }
}

#[derive(Clone)]
pub struct YummyState {
    #[allow(dead_code)]
    config: Arc<YummyConfig>,
    redis: r2d2::Pool<redis::Client>
}

impl YummyState {
    pub fn new(config: Arc<YummyConfig>, redis: r2d2::Pool<redis::Client>) -> Self {
        Self {
            config,
            redis
        }
    }

    #[tracing::instrument(name="ban_user_from_room", skip(self))]
    pub fn ban_user_from_room(&self, room_id: &RoomId, user_id: &UserId) -> Result<(), YummyStateError> {
        match self.redis.get() {
            Ok(mut redis) => {
                redis_result!(redis.sadd::<_, _, i32>(format!("{}room-banned:{}", self.config.redis_prefix, room_id.to_string()), user_id.to_string()));
                Ok(())
            },
            Err(_) => Err(YummyStateError::CacheCouldNotReaded)
        }
    }

    #[tracing::instrument(name="is_user_online", skip(self))]
    pub fn is_empty(&self) -> bool {
        match self.redis.get() {
            Ok(mut redis) => {
                let keys = redis_result!(redis::cmd("KEYS").arg(&format!("{}*", self.config.redis_prefix)).query::<Vec<String>>(&mut redis));
                keys.len() == 0
            },
            Err(_) => false
        }
    }

    #[tracing::instrument(name="is_user_online", skip(self))]
    pub fn is_user_online(&mut self, user_id: &UserId) -> bool {
        match self.redis.get() {
            Ok(mut redis) => redis_result!(redis.sismember(format!("{}online-users", self.config.redis_prefix), user_id.borrow().to_string())),
            Err(_) => false
        }
    }

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

    #[tracing::instrument(name="get_users_room_type", skip(self))]
    pub fn get_users_room_type(&mut self, session_id: &SessionId, room_id: &RoomId) -> Option<RoomUserType> {

        match self.redis.get() {
            Ok(mut redis) => match redis_result!(redis.hget(format!("{}room-sessions:{}", self.config.redis_prefix, room_id.to_string()), session_id.to_string())) {
                Some(1) => Some(RoomUserType::User),
                Some(2) => Some(RoomUserType::Moderator),
                Some(3) => Some(RoomUserType::Owner),
                _ => None
            },
            Err(_) => None
        }
    }

    #[tracing::instrument(name="set_users_room_type", skip(self))]
    pub fn set_users_room_type(&mut self, user_id: &UserId, room_id: &RoomId, user_type: RoomUserType) -> Result<(), YummyStateError> {
        match self.redis.get() {
            Ok(mut redis) => {
                let session_id = self.get_user_session_id(user_id, room_id)?;
                redis_result!(redis.hset::<_, _, _, i32>(format!("{}room-sessions:{}", self.config.redis_prefix, room_id.to_string()), session_id.to_string(), user_type as i32));
                Ok(())
            },
            Err(_) => Err(YummyStateError::CacheCouldNotReaded)
        }
    }

    #[tracing::instrument(name="is_session_online", skip(self))]
    pub fn is_session_online(&mut self, session_id: &SessionId) -> bool {
        match self.redis.get() {
            Ok(mut redis) => redis_result!(redis.hexists::<_, _, bool>(format!("{}session-user", self.config.redis_prefix), session_id.to_string())),
            Err(_) => false
        }
    }

    #[tracing::instrument(name="get_user_session_id", skip(self))]
    pub fn get_user_session_id(&self, user_id: &UserId, room_id: &RoomId) -> Result<SessionId, YummyStateError> {
        match self.redis.get() {
            Ok(mut redis) => {
                Ok(redis_result!(redis.hget::<_, _, SessionId>(format!("{}user-room:{}", self.config.redis_prefix, user_id.to_string()), room_id.to_string())))
            },
            Err(_) => Err(YummyStateError::CacheCouldNotReaded)
        }
    }

    #[tracing::instrument(name="set_user_type", skip(self))]
    pub fn set_user_type(&self, user_id: &UserId, user_type: UserType) {
        match self.redis.get() {
            Ok(mut redis) => redis_result!(redis.hset::<_, _, _, i32>(format!("{}users:{}", self.config.redis_prefix, user_id.to_string()), "type", i32::from(user_type))),
            Err(_) => 0
        };
    }

    #[tracing::instrument(name="set_user_name", skip(self))]
    pub fn set_user_name(&self, user_id: &UserId, name: String) {
        match self.redis.get() {
            Ok(mut redis) => redis_result!(redis.hset::<_, _, _, i32>(format!("{}users:{}", self.config.redis_prefix, user_id.to_string()), "name", name)),
            Err(_) => 0
        };
    }

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
                    .arg("type").arg(i32::from(user_type))
                    .arg("name").arg(name.unwrap_or_default())
                    .arg("loc").arg(&self.config.server_name)
                    .ignore()
                .query::<()>(&mut redis));
        }
        session_id
    }

    #[tracing::instrument(name="close_session", skip(self))]
    pub fn close_session(&mut self, user_id: &UserId, session_id: &SessionId) -> bool {
        
        match self.redis.get() {
            Ok(mut redis) => {
                let user_id_str = user_id.to_string();
                let session_id_str = session_id.to_string();

                let (remove_result,) = redis_result!(redis::pipe()
                    .atomic()
                    .cmd("HDEL").arg(format!("{}session-user", self.config.redis_prefix))
                        .arg(&session_id_str)
                        .ignore()
                    
                    .cmd("HGET").arg(format!("{}users:{}", self.config.redis_prefix, user_id_str))
                        .arg("room")
                        .ignore()

                    .cmd("DEL").arg(format!("{}users:{}", self.config.redis_prefix, user_id_str))
                        .ignore()

                    .cmd("DEL").arg(format!("{}user-jrequest:{}", self.config.redis_prefix, user_id_str))
                        .ignore()
                    
                    .cmd("SREM").arg(format!("{}online-users", self.config.redis_prefix))
                        .arg(&user_id_str)
                    .query::<(i32,)>(&mut redis));

                remove_result > 0
            },
            Err(_) => false
        }
    }

    #[tracing::instrument(name="get_user_rooms", skip(self))]
    pub fn get_user_rooms(&mut self, session_id: &SessionId) -> Option<Vec<RoomId>> {
        match self.redis.get() {
            Ok(mut redis) => Some(redis_result!(redis.smembers::<_, Vec<RoomId>>(format!("{}session-room:{}", self.config.redis_prefix, session_id.to_string())))),
            Err(_) => None
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(name="create_room", skip(self))]
    pub fn create_room(&self, room_id: &RoomId, insert_date: i32, name: Option<String>,  description: Option<String>, access_type: CreateRoomAccessType, max_user: usize, tags: Vec<String>, metas: Option<HashMap<String, MetaType<RoomMetaAccess>>>, join_request: bool) {
        if let Ok(mut redis) = self.redis.get() {
            let room_id = room_id.to_string();
            
            let mut pipes = &mut redis::pipe();
            pipes = pipes
                .atomic()
                .cmd("SADD").arg(format!("{}rooms", self.config.redis_prefix)).arg(&room_id).ignore()
                .cmd("HSET").arg(format!("{}room:{}", self.config.redis_prefix, &room_id))
                    .arg("max-user").arg(max_user)
                    .arg("user-len").arg(0_usize)
                    .arg("name").arg(name.unwrap_or_default())
                    .arg("access").arg(access_type as i32)
                    .arg("idate").arg(insert_date)
                    .arg("join").arg(join_request)
                    .arg("desc").arg(description.unwrap_or_default())
                    .ignore();

            if !tags.is_empty() {
                for tag in tags.iter() {
                    pipes = pipes.cmd("SADD").arg(format!("{}room-tag:{}", self.config.redis_prefix, &room_id)).arg(tag).ignore();
                    pipes = pipes.cmd("SADD").arg(format!("{}tag:{}", self.config.redis_prefix, &tag)).arg(&room_id).ignore();
                }
            }

            if let Some(metas) = metas {
                let room_meta_value = format!("{}room-meta-val:{}", self.config.redis_prefix, &room_id);
                let room_meta_type = format!("{}room-meta-type:{}", self.config.redis_prefix, &room_id);
                let room_meta_per = format!("{}room-meta-acc:{}", self.config.redis_prefix, &room_id);

                for (meta, value) in metas.iter() {
                    pipes = match value {
                        MetaType::Null => pipes,
                        MetaType::Number(value, per) => {
                            pipes.cmd("HSET").arg(&room_meta_value).arg(meta).arg(value).ignore();
                            pipes.cmd("HSET").arg(&room_meta_type).arg(meta).arg(1).ignore();
                            pipes.cmd("HSET").arg(&room_meta_per).arg(meta).arg(i32::from(*per)).ignore()
                        },
                        MetaType::String(value, per) => {
                            pipes.cmd("HSET").arg(&room_meta_value).arg(meta).arg(value).ignore();
                            pipes.cmd("HSET").arg(&room_meta_type).arg(meta).arg(2).ignore();
                            pipes.cmd("HSET").arg(&room_meta_per).arg(meta).arg(i32::from(*per)).ignore()
                        },
                        MetaType::Bool(value, per) => {
                            pipes.cmd("HSET").arg(&room_meta_value).arg(meta).arg(value).ignore();
                            pipes.cmd("HSET").arg(&room_meta_type).arg(meta).arg(3).ignore();
                            pipes.cmd("HSET").arg(&room_meta_per).arg(meta).arg(i32::from(*per)).ignore()
                        },
                        MetaType::List(value, per) => {
                            pipes.cmd("HSET").arg(&room_meta_value).arg(meta).arg(serde_json::to_string(value.deref()).unwrap_or_default()).ignore();
                            pipes.cmd("HSET").arg(&room_meta_type).arg(meta).arg(4).ignore();
                            pipes.cmd("HSET").arg(&room_meta_per).arg(meta).arg(i32::from(*per)).ignore()
                        }
                    }
                }
            }
            
            redis_result!(pipes.query::<()>(&mut redis));
        }
    }

    #[tracing::instrument(name="join_to_room_request", skip(self))]
    pub fn join_to_room_request(&mut self, room_id: &RoomId, user_id: &UserId, session_id: &SessionId, user_type: crate::model::RoomUserType) -> Result<(), YummyStateError> {

        let room_id = room_id.to_string();
        let room_info_key = format!("{}room:{}", self.config.redis_prefix, &room_id);
        match self.redis.get() {
            Ok(mut redis) => match redis_result!(redis.exists::<_, bool>(&room_info_key)) {
                true => {
                    let room_request_key = format!("{}room-request:{}", self.config.redis_prefix, &room_id);
                    let user_id = user_id.to_string();

                    let room_info = redis_result!(redis::cmd("HMGET")
                        .arg(format!("{}room:{}", self.config.redis_prefix, &room_id))
                        .arg("user-len")
                        .arg("max-user")
                        .query::<Vec<usize>>(&mut redis));

                    let user_len = room_info.first().copied().unwrap_or_default();
                    let max_user = room_info.get(1).copied().unwrap_or_default();

                    // If the max_user 0 or lower than users count, add to room
                    if max_user == 0 || max_user > user_len {
                        let is_member = redis_result!(redis.hexists(&room_request_key, &user_id));
    
                        // User alread in the room
                        if is_member {
                            return Err(YummyStateError::AlreadyRequested);
                        }

                        redis_result!(redis::pipe()
                            .atomic()
                            .cmd("HSET").arg(format!("{}user-jrequest:{}", self.config.redis_prefix, &user_id)).arg(&room_id).arg(session_id.to_string())
                            .cmd("HSET").arg(room_request_key).arg(&user_id).arg(user_type as i32).ignore()
                            .query::<()>(&mut redis));
                        Ok(())
                    } else {
                        Err(YummyStateError::RoomHasMaxUsers)
                    }
                }
                false => Err(YummyStateError::RoomNotFound)
            },
            Err(_) => Err(YummyStateError::CacheCouldNotReaded)
        }
    }

    #[tracing::instrument(name="join_to_room", skip(self))]
    pub fn join_to_room(&mut self, room_id: &RoomId, user_id: &UserId, session_id: &SessionId, user_type: crate::model::RoomUserType) -> Result<(), YummyStateError> {

        let room_info_key = format!("{}room:{}", self.config.redis_prefix, room_id.get());
        match self.redis.get() {
            Ok(mut redis) => match redis_result!(redis.exists::<_, bool>(&room_info_key)) {
                true => {
                    let session_id = session_id.to_string();
                    let user_id = user_id.to_string();
                    let room_id = room_id.to_string();
                    let room_sessions_key = format!("{}room-sessions:{}", self.config.redis_prefix, &room_id);

                    let room_info = redis_result!(redis::cmd("HMGET")
                        .arg(format!("{}room:{}", self.config.redis_prefix, &room_id))
                        .arg("user-len")
                        .arg("max-user")
                        .query::<Vec<usize>>(&mut redis));

                    let user_len = room_info.first().copied().unwrap_or_default();
                    let max_user = room_info.get(1).copied().unwrap_or_default();

                    // If the max_user 0 or lower than users count, add to room
                    if max_user == 0 || max_user > user_len {
                        let is_member = redis_result!(redis.hexists(&room_sessions_key, &session_id));
    
                        // User alread in the room
                        if is_member {
                            return Err(YummyStateError::UserAlreadInRoom);
                        }

                        redis_result!(redis::pipe()
                            .atomic()
                            .cmd("HSET").arg(format!("{}user-room:{}", self.config.redis_prefix, &user_id)).arg(&room_id).arg(&session_id).ignore()
                            .cmd("SADD").arg(format!("{}session-room:{}", self.config.redis_prefix, &session_id)).arg(&room_id)
                            .cmd("HINCRBY").arg(&room_info_key).arg("user-len").arg(1).ignore()
                            .cmd("HSET").arg(room_sessions_key).arg(&session_id).arg(user_type as i32).ignore()
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

    #[tracing::instrument(name="disconnect_from_room", skip(self))]
    pub fn disconnect_from_room(&mut self, room_id: &RoomId, user_id: &UserId, session_id: &SessionId) -> Result<bool, YummyStateError> {
        let room_removed: bool = match self.redis.get() {
            Ok(mut redis) => {
                let room_id = room_id.to_string();
                let user_id = user_id.to_string();
                let session_id = session_id.to_string();
                let room_info_key = format!("{}room:{}", self.config.redis_prefix, &room_id);
                let room_sessions_key = &format!("{}room-sessions:{}", self.config.redis_prefix, &room_id);

                let (user_len,) =  redis_result!(redis::pipe()
                    .atomic()
                    .cmd("SREM").arg(format!("{}session-room:{}", self.config.redis_prefix, &session_id)).arg(&room_id).ignore()
                    .cmd("HDEL").arg(room_sessions_key).arg(session_id).ignore()
                    .cmd("HDEL").arg(format!("{}user-room:{}", self.config.redis_prefix, &user_id)).arg(&room_id).ignore()
                    .cmd("HINCRBY").arg(&room_info_key).arg("user-len").arg(-1)
                    .query::<(i32,)>(&mut redis));
                    
                let no_user = user_len == 0;

                if no_user {
                    let (tags,) = redis_result!(redis::pipe()
                        .atomic()
                        .cmd("SREM").arg(format!("{}rooms", self.config.redis_prefix)).arg(&room_id).ignore()
                        .cmd("DEL").arg(room_sessions_key).ignore()
                        .cmd("DEL").arg(room_info_key).ignore()
                        .cmd("SMEMBERS").arg(format!("{}room-tag:{}", self.config.redis_prefix, &room_id))
                        .cmd("DEL").arg(format!("{}room-tag:{}", self.config.redis_prefix, room_id)).ignore()
                        .cmd("DEL").arg(format!("{}room-meta-val:{}", self.config.redis_prefix, room_id)).ignore()
                        .cmd("DEL").arg(format!("{}room-meta-type:{}", self.config.redis_prefix, room_id)).ignore()
                        .cmd("DEL").arg(format!("{}room-meta-acc:{}", self.config.redis_prefix, room_id)).ignore()
                        .cmd("DEL").arg(format!("{}room-request:{}", self.config.redis_prefix, room_id)).ignore()
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

    #[tracing::instrument(name="get_users_from_room", skip(self))]
    pub fn get_users_from_room(&mut self, room_id: &RoomId) -> Result<Vec<Arc<UserId>>, YummyStateError> {
        let room_id = room_id.get();
        match self.redis.get() {
            Ok(mut redis) => match redis_result!(redis.exists::<_, bool>(&format!("{}room-sessions:{}", self.config.redis_prefix, &room_id))) {
                true => {
                    let mut users = Vec::new();
                    let sessions : Vec<String> = redis_result!(redis.hkeys(&format!("{}room-sessions:{}", self.config.redis_prefix, &room_id)));
                    for session_id in sessions.into_iter() {
                                
                        let user_id: String = redis_result!(redis.hget::<_, _, String>(format!("{}session-user", self.config.redis_prefix), &session_id));
                        users.push(Arc::new(UserId::from(user_id)));
                    }

                    Ok(users)
                }
                false => Err(YummyStateError::RoomNotFound),
            },
            Err(_) => Err(YummyStateError::CacheCouldNotReaded)
        }
    }

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

    #[tracing::instrument(name="get_join_requests", skip(self))]
    pub fn get_join_requests(&self, room_id: &RoomId) -> Result<HashMap<Arc<UserId>, RoomUserType>, YummyStateError> {
        match self.redis.get() {
            Ok(mut redis) => {
                let users: HashMap<_, _> = redis_result!(redis.hgetall::<_, Vec<(UserId, RoomUserType)>>(format!("{}room-request:{}", self.config.redis_prefix, room_id.to_string())))
                    .into_iter()
                    .map(|(user_id, room_user_type)| (Arc::new(user_id), room_user_type))
                    .collect();
                Ok(users)
            },
            Err(_) => Err(YummyStateError::CacheCouldNotReaded)
        }
    }

    #[tracing::instrument(name="update_waiting_user_status", skip(self))]
    pub fn remove_user_from_waiting_list(&mut self, user_id: &UserId, room_id: &RoomId) -> Result<(SessionId, RoomUserType), YummyStateError> {
        match self.redis.get() {
            Ok(mut redis) => {
                let user_id = user_id.to_string();
                let room_id = room_id.to_string();
                
                let (session_id, room_user_type) = redis_result!(redis::pipe()
                    .cmd("HGET").arg(format!("{}user-jrequest:{}", self.config.redis_prefix, &user_id)).arg(&room_id)
                    .cmd("HDEL").arg(format!("{}user-jrequest:{}", self.config.redis_prefix, &user_id)).arg(&room_id).ignore()
                    .cmd("HGET").arg(format!("{}room-request:{}",  self.config.redis_prefix, &room_id)).arg(&user_id)
                    .cmd("HDEL").arg(format!("{}room-request:{}",  self.config.redis_prefix, &room_id)).arg(&user_id).ignore()
                    .query::<(SessionId, RoomUserType)>(&mut redis));

                Ok((session_id, room_user_type))
            },
            Err(_) => Err(YummyStateError::CacheCouldNotReaded)
        }
    }

    #[tracing::instrument(name="is_user_banned_from_room", skip(self))]
    pub fn is_user_banned_from_room(&self, room_id: &RoomId, user_id: &UserId) -> Result<bool, YummyStateError> {
        match self.redis.get() {
            Ok(mut redis) => Ok(redis_result!(redis.sismember(format!("{}room-banned:{}", self.config.redis_prefix, room_id.to_string()), user_id.to_string()))),
            Err(_) => Err(YummyStateError::CacheCouldNotReaded)
        }
    }

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
                        RoomInfoTypeVariant::Description => request = request.arg("desc"),
                        RoomInfoTypeVariant::Users => request = request.arg("users"),
                        RoomInfoTypeVariant::MaxUser => request = request.arg("max-user"),
                        RoomInfoTypeVariant::UserLength => request = request.arg("user-len"),
                        RoomInfoTypeVariant::AccessType => request = request.arg("access"),
                        RoomInfoTypeVariant::JoinRequest => request = request.arg("join"),
                        RoomInfoTypeVariant::InsertDate => request = request.arg("idate"),
                        RoomInfoTypeVariant::Tags => request = request.arg("tags"), // Dummy data, dont remove
                        RoomInfoTypeVariant::BannedUsers => request = request.arg("bu"), // Dummy data, dont remove
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
                        RoomInfoTypeVariant::Description => {
                            let description: String = FromRedisValue::from_redis_value(&room_info).unwrap_or_default();
                            result.items.push(RoomInfoType::Description(if description.is_empty() { None } else { Some(description) }));
                        },
                        RoomInfoTypeVariant::Users => {

                            // This request is slow compare to other. We should change it to lua script to increase performance
                            let mut user_infos = Vec::new();
                            let sessions = redis_result!(redis.hgetall::<_, HashMap<SessionId, RoomUserType>>(format!("{}room-sessions:{}", self.config.redis_prefix, &room_id)));

                            for (session_id, user_type) in sessions.into_iter() {
                                
                                let user_id: String = redis_result!(redis.hget::<_, _, String>(format!("{}session-user", self.config.redis_prefix), session_id.to_string()));
                                let name = redis_result!(redis.hget::<_, _, String>(format!("{}users:{}", self.config.redis_prefix, &user_id), "name"));
                                user_infos.push(RoomUserInformation {
                                    name: if name.is_empty() { None } else { Some(name) },
                                    user_id: Arc::new(UserId::from(user_id)),
                                    user_type
                                })
                            }
                            result.items.push(RoomInfoType::Users(user_infos));
                        },
                        RoomInfoTypeVariant::BannedUsers => {
                            let users = redis_result!(redis.hgetall::<_, HashSet<UserId>>(format!("{}room-banned:{}", self.config.redis_prefix, &room_id)));
                            result.items.push(RoomInfoType::BannedUsers(users));
                        },
                        RoomInfoTypeVariant::AccessType => result.items.push(RoomInfoType::AccessType(FromRedisValue::from_redis_value(&room_info).unwrap_or_default())),
                        RoomInfoTypeVariant::JoinRequest => result.items.push(RoomInfoType::JoinRequest(FromRedisValue::from_redis_value(&room_info).unwrap_or_default())),
                        RoomInfoTypeVariant::InsertDate => result.items.push(RoomInfoType::InsertDate(FromRedisValue::from_redis_value(&room_info).unwrap_or_default())),
                        RoomInfoTypeVariant::MaxUser => result.items.push(RoomInfoType::MaxUser(FromRedisValue::from_redis_value(&room_info).unwrap_or_default())),
                        RoomInfoTypeVariant::UserLength => result.items.push(RoomInfoType::UserLength(FromRedisValue::from_redis_value(&room_info).unwrap_or_default())),
                        RoomInfoTypeVariant::Tags => {
                            let tags = redis_result!(redis.smembers::<_, Vec<String>>(format!("{}room-tag:{}", self.config.redis_prefix, &room_id)));
                            result.items.push(RoomInfoType::Tags(tags));
                        },
                        RoomInfoTypeVariant::Metas => {
                            let access_level = i32::from(access_level);

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

                            if !keys.is_empty() {
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
                                        4 => {
                                            let value: String = FromRedisValue::from_redis_value(&value).unwrap_or_default();
                                            MetaType::List(Box::new(serde_json::from_str(&value).unwrap_or_default()), RoomMetaAccess::from(access))
                                        },
                                        _ => MetaType::Number(FromRedisValue::from_redis_value(&value).unwrap_or_default(), RoomMetaAccess::from(access)),
                                    };
    
                                    metas.insert(key, value);
                                }
    
                                result.items.push(RoomInfoType::Metas(metas));
                            } else {
                                result.items.push(RoomInfoType::Metas(HashMap::new()));
                            }
                        },
                    };
                }

                Ok(result)
            },
            Err(_) => Err(YummyStateError::CacheCouldNotReaded)
        }
    }

    #[tracing::instrument(name="set_room_info", skip(self))]
    pub fn set_room_info(&self, room_id: &RoomId, query: Vec<RoomInfoType>) {
        if query.is_empty() {
            return;
        }

        if let Ok(mut redis) = self.redis.get() {
            let mut command = &mut redis::pipe();
            let room_id = room_id.to_string();

            for item in query.into_iter() {
                match item {
                    RoomInfoType::RoomName(name) => command = command.cmd("HSET").arg(format!("{}room:{}", self.config.redis_prefix, &room_id)).arg("name").arg(name.unwrap_or_default()).ignore(),
                    RoomInfoType::Description(description) => command = command.cmd("HSET").arg(format!("{}room:{}", self.config.redis_prefix, &room_id)).arg("desc").arg(description.unwrap_or_default()).ignore(),
                    RoomInfoType::Users(_) => (),
                    RoomInfoType::BannedUsers(_) => (),
                    RoomInfoType::MaxUser(max_user) => command = command.cmd("HSET").arg(format!("{}room:{}", self.config.redis_prefix, &room_id)).arg("max-user").arg(max_user).ignore(),
                    RoomInfoType::UserLength(_) => (),
                    RoomInfoType::AccessType(access_type) => command = command.cmd("HSET").arg(format!("{}room:{}", self.config.redis_prefix, &room_id)).arg("access").arg(i32::from(access_type)).ignore(),
                    RoomInfoType::JoinRequest(join_request) => command = command.cmd("HSET").arg(format!("{}room:{}", self.config.redis_prefix, &room_id)).arg("join").arg(i32::from(join_request)).ignore(),
                    RoomInfoType::Tags(tags) => {
                        
                        // Remove old tags
                        let saved_tags = redis_result!(redis.smembers::<_, Vec<String>>(format!("{}room-tag:{}", self.config.redis_prefix, room_id)));
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
                    RoomInfoType::Metas(metas) => {
                        command = command
                            .cmd("DEL").arg(format!("{}room-meta-val:{}", self.config.redis_prefix, &room_id)).ignore()
                            .cmd("DEL").arg(format!("{}room-meta-type:{}", self.config.redis_prefix, &room_id)).ignore()
                            .cmd("DEL").arg(format!("{}room-meta-acc:{}", self.config.redis_prefix, &room_id)).ignore();

                        let room_meta_value = format!("{}room-meta-val:{}", self.config.redis_prefix, &room_id);
                        let room_meta_type = format!("{}room-meta-type:{}", self.config.redis_prefix, &room_id);
                        let room_meta_per = format!("{}room-meta-acc:{}", self.config.redis_prefix, &room_id);
        
                        for (meta, value) in metas.iter() {
                            command = match value {
                                MetaType::Null => command,
                                MetaType::Number(value, per) => {
                                    command.cmd("HSET").arg(&room_meta_value).arg(meta).arg(value).ignore();
                                    command.cmd("HSET").arg(&room_meta_type).arg(meta).arg(1).ignore();
                                    command.cmd("HSET").arg(&room_meta_per).arg(meta).arg(i32::from(*per)).ignore()
                                },
                                MetaType::String(value, per) => {
                                    command.cmd("HSET").arg(&room_meta_value).arg(meta).arg(value).ignore();
                                    command.cmd("HSET").arg(&room_meta_type).arg(meta).arg(2).ignore();
                                    command.cmd("HSET").arg(&room_meta_per).arg(meta).arg(i32::from(*per)).ignore()
                                },
                                MetaType::Bool(value, per) => {
                                    command.cmd("HSET").arg(&room_meta_value).arg(meta).arg(value).ignore();
                                    command.cmd("HSET").arg(&room_meta_type).arg(meta).arg(3).ignore();
                                    command.cmd("HSET").arg(&room_meta_per).arg(meta).arg(i32::from(*per)).ignore()
                                },
                                MetaType::List(value, per) => {
                                    command.cmd("HSET").arg(&room_meta_value).arg(meta).arg(serde_json::to_string(value.deref()).unwrap_or_default()).ignore();
                                    command.cmd("HSET").arg(&room_meta_type).arg(meta).arg(4).ignore();
                                    command.cmd("HSET").arg(&room_meta_per).arg(meta).arg(i32::from(*per)).ignore()
                                }
                            }
                        }
                    },
                };
            }

            redis_result!(command.query::<()>(&mut redis));
        }
    }

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
                            RoomInfoTypeVariant::Description => command = command.arg("desc"),
                            RoomInfoTypeVariant::Users => command = command.arg("users"),
                            RoomInfoTypeVariant::MaxUser => command = command.arg("max-user"),
                            RoomInfoTypeVariant::UserLength => command = command.arg("user-len"),
                            RoomInfoTypeVariant::AccessType => command = command.arg("access"),
                            RoomInfoTypeVariant::InsertDate => command = command.arg("idate"),
                            RoomInfoTypeVariant::JoinRequest => command = command.arg("join"),
                            RoomInfoTypeVariant::Tags => command = command.arg("tags"), // Dummy data, dont remove
                            RoomInfoTypeVariant::BannedUsers => command = command.arg("bu"), // Dummy data, dont remove
                            RoomInfoTypeVariant::Metas => command = command.arg("metas"), // Dummy data, dont remove
                        };
                    }
                }

                let room_results = redis_result!(command.query::<Vec<redis::Value>>(&mut redis));

                for (room_id, room_result) in rooms.into_iter().zip(room_results.into_iter()) {
                    let room_id_str = room_id;
                    let room_id = RoomId::from(uuid::Uuid::parse_str(&room_id_str).unwrap_or_default());

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
                            RoomInfoTypeVariant::Description => {
                                let description: String = FromRedisValue::from_redis_value(redis_value).unwrap_or_default();
                                room_info.items.push(RoomInfoType::Description(if description.is_empty() { None } else { Some(description) }));
                            },
                            RoomInfoTypeVariant::Users => {
    
                                // This request is slow compare to other. We should change it to lua script to increase performance
                                let mut user_infos = Vec::new();
                                let users = redis_result!(redis.hgetall::<_, HashMap<UserId, RoomUserType>>(format!("{}room-sessions:{}", self.config.redis_prefix, &room_id_str)));
                                for (session_id, user_type) in users.into_iter() {
                                    let user_id: String = redis_result!(redis.hget::<_, _, String>(format!("{}session-user", self.config.redis_prefix), session_id.to_string()));
                                    
                                    let name = redis_result!(redis.hget::<_, _, String>(format!("{}users:{}", self.config.redis_prefix, &user_id.to_string()), "name"));
                                    user_infos.push(RoomUserInformation {
                                        name: if name.is_empty() { None } else { Some(name) },
                                        user_id: Arc::new(UserId::from(user_id)),
                                        user_type
                                    })
                                }
                                room_info.items.push(RoomInfoType::Users(user_infos));
                            },
                            RoomInfoTypeVariant::BannedUsers => {
                                let users = redis_result!(redis.hgetall::<_, HashSet<UserId>>(format!("{}room-banned:{}", self.config.redis_prefix, &room_id_str)));
                                room_info.items.push(RoomInfoType::BannedUsers(users));
                            },
                            RoomInfoTypeVariant::AccessType => room_info.items.push(RoomInfoType::AccessType(FromRedisValue::from_redis_value(redis_value).unwrap_or_default())),
                            RoomInfoTypeVariant::InsertDate => room_info.items.push(RoomInfoType::InsertDate(FromRedisValue::from_redis_value(redis_value).unwrap_or_default())),
                            RoomInfoTypeVariant::JoinRequest => room_info.items.push(RoomInfoType::JoinRequest(FromRedisValue::from_redis_value(redis_value).unwrap_or_default())),
                            RoomInfoTypeVariant::MaxUser => room_info.items.push(RoomInfoType::MaxUser(FromRedisValue::from_redis_value(redis_value).unwrap_or_default())),
                            RoomInfoTypeVariant::UserLength => room_info.items.push(RoomInfoType::UserLength(FromRedisValue::from_redis_value(redis_value).unwrap_or_default())),
                            RoomInfoTypeVariant::Tags => {
                                let tags = redis_result!(redis.smembers::<_, Vec<String>>(format!("{}room-tag:{}", self.config.redis_prefix, &room_id_str)));
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
}
