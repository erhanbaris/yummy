use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::borrow::Borrow;
use std::sync::atomic::AtomicUsize;

use general::model::*;
use general::meta::*;
use general::config::YummyConfig;

use crate::cache::{YummyCache, YummyCacheResource};
use crate::state_resource::UserMetaInformation;

use super::*;
use super::resource::YummyCacheResourceFactory;

#[derive(Clone)]
pub struct ConnectionInfo {
    pub user_id: Arc<UserId>,
    pub room_user_type: RoomUserType
}

#[derive(Default)]
pub struct ConnectionResource;

impl YummyCacheResource for ConnectionResource {
    type K=SessionId;
    type V=ConnectionInfo;
    
    fn get(&self, _: &Self::K) -> anyhow::Result<Option<Self::V>> { Ok(None) }

    fn set(&self, _: &Self::K, _: &Self::V) -> anyhow::Result<()> { Ok(()) }
}

struct RoomState {
    pub name: Option<String>,
    pub description: Option<String>,
    pub access_type: CreateRoomAccessType,
    pub max_user: usize,
    pub tags: Vec<String>,
    pub insert_date: i32,
    pub join_request: bool,
    pub connection_count: AtomicUsize,
    pub connections: YummyCache<SessionId, ConnectionInfo>,
    pub banned_users: HashSet<UserId>,
    pub metas: HashMap<String, MetaType<RoomMetaAccess>>,
    pub join_requests: HashMap<SessionId, RoomUserType>,
}


#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct UserState {
    pub user_id: UserId,
    pub name: Option<String>,
    pub user_type: UserType,

    pub sessions: std::collections::HashSet<SessionId>,
    pub join_requests: std::collections::HashMap<RoomId, SessionId>,
    pub joined_rooms: std::collections::HashMap<RoomId, SessionId>,
}

#[derive(Clone)]
pub struct YummyState {
    #[allow(dead_code)]
    config: Arc<YummyConfig>,
    users: Arc<parking_lot::Mutex<std::collections::HashMap<UserId, UserState>>>,
    rooms: Arc<parking_lot::Mutex<std::collections::HashMap<RoomId, RoomState>>>,
    session_to_users: Arc<parking_lot::Mutex<std::collections::HashMap<SessionId, Arc<UserId>>>>,
    session_to_room: Arc<parking_lot::Mutex<std::collections::HashMap<SessionId, std::collections::HashSet<RoomId>>>>,
    user_informations: Arc<YummyCache<UserId, UserInformationModel>>,
    user_metas: Arc<YummyCache<UserId, Vec<UserMetaInformation>>>
}

impl YummyState {
    pub fn new(config: Arc<YummyConfig>, resource_factory: Box<dyn YummyCacheResourceFactory>) -> Self {
        let user_informations = YummyCache::new(config.clone(), resource_factory.user_information());
        let user_metas = YummyCache::new(config.clone(), resource_factory.user_metas());

        Self {
            config: config.clone(),

            users: Arc::new(parking_lot::Mutex::default()),
            rooms: Arc::new(parking_lot::Mutex::default()),
            session_to_users: Arc::new(parking_lot::Mutex::default()),
            session_to_room: Arc::new(parking_lot::Mutex::default()),
            user_informations: Arc::new(user_informations),
            user_metas: Arc::new(user_metas),
        }
    }

    pub fn get_user_information(&self, user_id: &UserId, access: UserMetaAccess) -> Result<Option<UserInformationModel>, YummyStateError> {
        match self.user_informations.get(user_id)? {
            Some(mut result) => {
                let access = access as i32;
                result.metas = result.metas.map(|metas| metas.into_iter().filter(|(_, value)| value.get_access_level() as i32 <= access).collect());
                Ok(Some(result))
            },
            None => Ok(None)
        }
    }

    pub fn update_user_information(&self, user_id: &UserId, informations: UserInformationModel) -> Result<(), YummyStateError> {
        self.user_informations.set(user_id, informations)?;
        Ok(())
    }

    pub fn get_user_meta(&self, user_id: &UserId, access: UserMetaAccess) -> Result<Vec<UserMetaInformation>, YummyStateError> {
        match self.user_metas.get(user_id)? {
            Some(mut metas) => {
                let access_type = access as i32;
                metas = metas
                    .into_iter()
                    .filter(|meta| meta.meta.get_access_level() as i32 <= access_type)
                    .collect::<Vec<UserMetaInformation>>();
                Ok(metas)
            },
            None => Ok(Vec::new())
        }
    }
    
    #[tracing::instrument(name="ban_user_from_room", skip(self))]
    pub fn ban_user_from_room(&self, room_id: &RoomId, user_id: &UserId) -> Result<(), YummyStateError> {
        match self.rooms.lock().get_mut(room_id) {
            Some(room) => {
                room.banned_users.insert(user_id.clone());
                Ok(())
            },
            None => Err(YummyStateError::RoomNotFound)
        }
    }
    
    #[tracing::instrument(name="is_empty", skip(self))]
    pub fn is_empty(&self) -> bool {
        self.rooms.lock().len() == 0 &&
            self.users.lock().len() == 0 &&
            self.session_to_room.lock().len() == 0 &&
            self.session_to_users.lock().len() == 0
    }

    #[tracing::instrument(name="is_user_online", skip(self))]
    pub fn is_user_online(&self, user_id: &UserId) -> bool {
        self.users.lock().contains_key(user_id)
    }

    #[tracing::instrument(name="get_user_type", skip(self))]
    pub fn get_user_type(&mut self, user_id: &UserId) -> Option<UserType> {
        self.users.lock().get(user_id).map(|user| user.user_type)
    }
    
    
    #[tracing::instrument(name="get_users_room_type", skip(self))]
    pub fn get_users_room_type(&mut self, session_id: &SessionId, room_id: &RoomId) -> anyhow::Result<Option<RoomUserType>> {
        match self.rooms.lock().get(room_id) {
            Some(room) => Ok(room.connections.get(session_id)?.map(|connection| connection.room_user_type.clone())),
            None => Ok(None)
        }
    }

    #[tracing::instrument(name="set_users_room_type", skip(self))]
    pub fn set_users_room_type(&mut self, user_id: &UserId, room_id: &RoomId, user_type: RoomUserType) -> Result<(), YummyStateError> {
        let session_id = match self.users.lock().get(user_id) {
            Some(user) => match user.joined_rooms.get(room_id) {
                Some(session_id) => session_id.clone(),
                None => return Err(YummyStateError::UserNotFound)
            },
            None => return Err(YummyStateError::UserNotFound)
        };

        if let Some(room) = self.rooms.lock().get_mut(room_id) {
            match room.connections.get(&session_id)? {
                Some(mut user) => {
                    user.room_user_type = user_type;
                    room.connections.set(&session_id, user)?;
                    room.connection_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                },
                None => return Err(YummyStateError::UserNotFound)
            };
        }

        Ok(())
    }

    #[tracing::instrument(name="is_session_online", skip(self))]
    pub fn is_session_online<T: Borrow<SessionId> + std::fmt::Debug>(&self, session_id: T) -> bool {
        self.session_to_users.lock().contains_key(session_id.borrow())
    }

    #[tracing::instrument(name="new_session", skip(self))]
    pub fn new_session(&self, user_id: &UserId, name: Option<String>, user_type: UserType) -> SessionId {
        use std::collections::HashSet;

        let session_id = SessionId::new();
        self.session_to_users.lock().insert(session_id.clone(), Arc::new(user_id.clone()));

        let mut users = self.users.lock();

        match users.get_mut(user_id) {
            Some(user) => {
                user.sessions.insert(session_id.clone());
            },
            None => {
                users.insert(user_id.clone(), UserState {
                    user_id: user_id.clone(),
                    name,
                    sessions: HashSet::from([session_id.clone()]),
                    user_type,
                    join_requests: HashMap::default(),
                    joined_rooms: HashMap::default()
                });
            }
        }
        session_id
    }

    #[tracing::instrument(name="set_user_type", skip(self))]
    pub fn set_user_type(&self, user_id: &UserId, user_type: UserType) {
        if let Some(user) = self.users.lock().get_mut(user_id) {
            user.user_type = user_type
        }
    }

    #[tracing::instrument(name="set_user_name", skip(self))]
    pub fn set_user_name(&self, user_id: &UserId, name: String) {
        if let Some(user) = self.users.lock().get_mut(user_id) {
            user.name = Some(name)
        }
    }

    #[tracing::instrument(name="close_session", skip(self))]
    pub fn close_session(&self, user_id: &UserId, session_id: &SessionId) -> bool {
        let user_id = self.session_to_users.lock().remove(session_id);

        match user_id {
            Some(user_id) => {
                let session_count = match self.users.lock().get_mut(&user_id) {
                    Some(user) => {
                        user.sessions.remove(session_id);
                        user.sessions.len()
                    }
                    None => 0
                };

                if session_count == 0 {
                    self.users.lock().remove(&user_id);
                }
                
                true
            },
            None => false
        }
    }

    #[tracing::instrument(name="get_user_rooms", skip(self))]
    pub fn get_user_rooms(&self, session_id: &SessionId) -> Option<Vec<RoomId>> {
        self.session_to_room.lock().get(session_id).map(|rooms| rooms.iter().cloned().collect::<Vec<_>>())
    }

    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(name="create_room", skip(self))]
    pub fn create_room(&self, room_id: &RoomId, insert_date: i32, name: Option<String>, description: Option<String>, access_type: CreateRoomAccessType, max_user: usize, tags: Vec<String>, metas: Option<HashMap<String, MetaType<RoomMetaAccess>>>, join_request: bool) {
        use std::collections::HashMap;

        self.rooms.lock().insert(*room_id, RoomState {
            max_user,
            insert_date,
            connection_count: AtomicUsize::new(0),
            connections: YummyCache::new(self.config.clone(), Box::new(ConnectionResource::default())),
            tags,
            name,
            description,
            access_type,
            metas: metas.unwrap_or_default(),
            join_request,
            join_requests: HashMap::default(),
            banned_users: HashSet::default(),
        });
    }

    #[tracing::instrument(name="join_to_room_request", skip(self))]
    pub fn join_to_room_request(&mut self, room_id: &RoomId, user_id: &UserId, session_id: &SessionId, user_type: RoomUserType) -> Result<(), YummyStateError> {
        match self.rooms.lock().get_mut(room_id.borrow()) {
            Some(room) => {

                if room.connections.contains(session_id) {
                    return Err(YummyStateError::UserAlreadInRoom);
                }
                
                let users_len = room.connection_count.load(std::sync::atomic::Ordering::Relaxed);

                // If the max_user 0 or lower than users count, add to room
                if room.max_user == 0 || room.max_user > users_len {
                    let inserted = room.join_requests.insert(session_id.clone(), user_type);

                    if inserted.is_some() {
                        return Err(YummyStateError::AlreadyRequested)
                    }

                    match self.users.lock().get_mut(user_id) {
                        Some(user) => user.join_requests.insert(*room_id, session_id.clone()),
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

    #[tracing::instrument(name="update_waiting_user_status", skip(self))]
    pub fn remove_user_from_waiting_list(&mut self, user_id: &UserId, room_id: &RoomId) -> Result<(SessionId, RoomUserType), YummyStateError> {
        match self.rooms.lock().get_mut(room_id.borrow()) {
            Some(room) => {

                let session_id = match self.users.lock().get_mut(user_id) {
                    Some(user) => user.join_requests.remove(room_id),
                    None => return Err(YummyStateError::UserNotFound)
                };

                let session_id = match session_id {
                    Some(session_id) => session_id,
                    None => return Err(YummyStateError::UserNotFound)
                };

                let room_user_type = match room.join_requests.remove(&session_id) {
                    Some(room_user_type) => room_user_type,
                    None => return Err(YummyStateError::UserNotFound)
                };

                Ok((session_id, room_user_type))
            },
            None => Err(YummyStateError::RoomNotFound)
        }
    }

    #[tracing::instrument(name="join_to_room", skip(self))]
    pub fn join_to_room(&mut self, room_id: &RoomId, user_id: &UserId, session_id: &SessionId, room_user_type: RoomUserType) -> Result<(), YummyStateError> {
        match self.rooms.lock().get_mut(room_id.borrow()) {
            Some(room) => {

                let users_len = room.connection_count.load(std::sync::atomic::Ordering::Relaxed);

                // If the max_user 0 or lower than users count, add to room
                if room.max_user == 0 || room.max_user > users_len {

                    // User alread in the room
                    if room.connections.contains(session_id) {
                        return Err(YummyStateError::UserAlreadInRoom);
                    }

                    room.connection_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    room.connections.set(&session_id, ConnectionInfo { user_id: Arc::new(user_id.clone()), room_user_type })?;
                    
                    let mut user_to_room = self.session_to_room.lock();
                    match user_to_room.get_mut(session_id) {
                        Some(user_to_room) => {
                            user_to_room.insert(*room_id);
                        },
                        None => {
                            user_to_room.insert(session_id.clone(), std::collections::HashSet::from([*room_id]));
                        }
                    };

                    if let Some(user) = self.users.lock().get_mut(user_id) {
                        user.joined_rooms.insert(*room_id, session_id.clone());
                    }

                    Ok(())
                } else {
                    Err(YummyStateError::RoomHasMaxUsers)
                }
            }
            None => Err(YummyStateError::RoomNotFound)
        }
    }

    pub fn get_user_session_id(&self, user_id: &UserId, room_id: &RoomId) -> Result<SessionId, YummyStateError> {
        match self.users.lock().get(user_id) {
            Some(user) => match user.joined_rooms.get(room_id) {
                Some(session_id) => Ok(session_id.clone()),
                None => Err(YummyStateError::RoomNotFound),
            },
            None => Err(YummyStateError::UserNotFound)
        }
    }

    #[tracing::instrument(name="disconnect_from_room", skip(self))]
    pub fn disconnect_from_room(&self, room_id: &RoomId, user_id: &UserId, session_id: &SessionId) -> Result<bool, YummyStateError> {
        let mut rooms = self.rooms.lock();
        let room_removed = match rooms.get_mut(room_id.borrow()) {
            Some(room) => {
                
                // Remove room from user
                match self.users.lock().get_mut(user_id) {
                    Some(user) => {
                        if user.joined_rooms.remove(room_id) .is_none(){

                            // User did not joined to room
                            return Err(YummyStateError::UserCouldNotFoundInRoom);
                        }
                    },
                    None => return Err(YummyStateError::UserNotFound)
                };

                let mut session_to_room = self.session_to_room.lock();
                let room_count = match session_to_room.get_mut(session_id) {
                    Some(rooms) => {
                        rooms.remove(room_id);
                        rooms.len()
                    },
                    None => 0
                };

                if room_count == 0 {
                    session_to_room.remove(session_id);
                }

                let user_removed = room.connections.contains(session_id);
                match user_removed {
                    true => {
                        room.connections.remove(session_id);
                        let previous_value = room.connection_count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                        Ok(previous_value - 1 == 0)
                    },
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

    #[tracing::instrument(name="get_users_from_room", skip(self))]
    pub fn get_users_from_room(&self, room_id: &RoomId) -> Result<Vec<Arc<UserId>>, YummyStateError> {
        match self.rooms.lock().get(room_id) {
            Some(room) => {
                Ok(room.connections
                    .iter()
                    .map(|(_, connection)| connection.user_id.clone())
                    .collect::<Vec<_>>())
            }
            None => Err(YummyStateError::RoomNotFound)
        }
    }

    #[tracing::instrument(name="get_user_location", skip(self))]
    pub fn get_user_location(&self, user_id: &UserId) -> Option<String> {
        None
    }

    #[tracing::instrument(name="get_join_requests", skip(self))]
    pub fn get_join_requests(&self, room_id: &RoomId) -> Result<HashMap<Arc<UserId>, RoomUserType>, YummyStateError> {
        match self.rooms.lock().get(room_id) {
            Some(room) => {
                let mut value = HashMap::new();

                let session_to_user = self.session_to_users.lock();

                for (session_id, room_user_type) in room.join_requests.iter() {
                    if let Some(user_id) = session_to_user.get(session_id) {
                        value.insert(user_id.clone(), room_user_type.clone());
                    }
                }

                Ok(value)
            },
            None => Err(YummyStateError::RoomNotFound)
        }
    }

    #[tracing::instrument(name="is_user_banned_from_room", skip(self))]
    pub fn is_user_banned_from_room(&self, room_id: &RoomId, user_id: &UserId) -> Result<bool, YummyStateError> {
        match self.rooms.lock().get(room_id) {
            Some(room) => Ok(room.banned_users.contains(user_id)),
            None => Err(YummyStateError::RoomNotFound)
        }
    }

    #[tracing::instrument(name="get_room_info", skip(self))]
    pub fn get_room_info(&self, room_id: &RoomId, access_level: RoomMetaAccess, query: &[RoomInfoTypeVariant]) -> Result<RoomInfoTypeCollection, YummyStateError> {
        let mut result = RoomInfoTypeCollection::default();
        match self.rooms.lock().get(room_id) {
            Some(room) => {
                room.connections.sync();

                for item in query.iter() {
                    let item = match item {
                        RoomInfoTypeVariant::InsertDate => RoomInfoType::InsertDate(room.insert_date),
                        RoomInfoTypeVariant::MaxUser => RoomInfoType::MaxUser(room.max_user),
                        RoomInfoTypeVariant::RoomName => RoomInfoType::RoomName(room.name.clone()),
                        RoomInfoTypeVariant::Description => RoomInfoType::Description(room.description.clone()),
                        RoomInfoTypeVariant::UserLength => RoomInfoType::UserLength(room.connection_count.load(std::sync::atomic::Ordering::Relaxed)),
                        RoomInfoTypeVariant::AccessType => RoomInfoType::AccessType(room.access_type.clone()),
                        RoomInfoTypeVariant::JoinRequest => RoomInfoType::JoinRequest(room.join_request),
                        RoomInfoTypeVariant::Users => {
                            let mut users = Vec::new();
                            let user_cache = self.users.lock();

                            for (_, connection_info) in room.connections.iter() {
                                let name = match user_cache.get(connection_info.user_id.deref()) {
                                    Some(user) => user.name.clone(),
                                    None => None
                                };
                                users.push(RoomUserInformation {
                                    user_id: connection_info.user_id.clone(),
                                    name,
                                    user_type: connection_info.room_user_type.clone()
                                });
                            }

                            RoomInfoType::Users(users)
                        },
                        RoomInfoTypeVariant::BannedUsers => RoomInfoType::BannedUsers(room.banned_users.clone()),
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

    #[tracing::instrument(name="set_room_info", skip(self))]
    pub fn set_room_info(&self, room_id: &RoomId, query: Vec<RoomInfoType>) {
        if let Some(room) = self.rooms.lock().get_mut(room_id) {
            for item in query.into_iter() {
                match item {
                    RoomInfoType::RoomName(name) => room.name = name,
                    RoomInfoType::Description(description) => room.description = description,
                    RoomInfoType::Users(_) => (),
                    RoomInfoType::MaxUser(max_user) => room.max_user = max_user,
                    RoomInfoType::UserLength(_) => (),
                    RoomInfoType::AccessType(access_type) => room.access_type = access_type,
                    RoomInfoType::JoinRequest(join_request) => room.join_request = join_request,
                    RoomInfoType::Tags(tags) => room.tags = tags,
                    RoomInfoType::Metas(metas) => room.metas = metas,
                    RoomInfoType::BannedUsers(banned_users) => room.banned_users = banned_users,
                    RoomInfoType::InsertDate(_) => (),
                };
            }
        }
    }

    #[tracing::instrument(name="get_rooms", skip(self))]
    pub fn get_rooms(&self, tag: &Option<String>, query: &[RoomInfoTypeVariant]) -> Result<Vec<RoomInfoTypeCollection>, YummyStateError> {
        let mut result = Vec::default();
        let rooms = self.rooms.lock();
        let rooms = match tag {
            Some(tag) => rooms.iter().filter(|item| item.1.tags.contains(&tag)).collect::<Vec<_>>(),
            None => rooms.iter().collect::<Vec<_>>()
        };

        for (room_id, room_state) in rooms.into_iter() {
            let mut room_info = RoomInfoTypeCollection {
                room_id: Some(*room_id),
                .. Default::default()
            };

            room_state.connections.sync();

            for item in query.iter() {
                match item {
                    RoomInfoTypeVariant::InsertDate => room_info.items.push(RoomInfoType::InsertDate(room_state.insert_date)),
                    RoomInfoTypeVariant::MaxUser => room_info.items.push(RoomInfoType::MaxUser(room_state.max_user)),
                    RoomInfoTypeVariant::JoinRequest => room_info.items.push(RoomInfoType::JoinRequest(room_state.join_request)),
                    RoomInfoTypeVariant::RoomName => room_info.items.push(RoomInfoType::RoomName(room_state.name.clone())),
                    RoomInfoTypeVariant::Description => room_info.items.push(RoomInfoType::Description(room_state.description.clone())),
                    RoomInfoTypeVariant::UserLength => room_info.items.push(RoomInfoType::UserLength(room_state.connection_count.load(std::sync::atomic::Ordering::Relaxed))),
                    RoomInfoTypeVariant::AccessType => room_info.items.push(RoomInfoType::AccessType(room_state.access_type.clone())),
                    RoomInfoTypeVariant::Users => {                        
                        let mut users = Vec::new();
                        let user_cache = self.users.lock();

                        for (_, connection_info) in room_state.connections.iter() {
                            let name = match user_cache.get(connection_info.user_id.deref()) {
                                Some(user) => user.name.clone(),
                                None => None
                            };
                            users.push(RoomUserInformation {
                                user_id: connection_info.user_id.clone(),
                                name,
                                user_type: connection_info.room_user_type.clone()
                            });
                        }

                        room_info.items.push(RoomInfoType::Users(users))
                    },
                    RoomInfoTypeVariant::BannedUsers => room_info.items.push(RoomInfoType::BannedUsers(room_state.banned_users.clone())),
                    RoomInfoTypeVariant::Tags => room_info.items.push(RoomInfoType::Tags(room_state.tags.clone())),
                    RoomInfoTypeVariant::Metas => room_info.items.push(RoomInfoType::Metas(room_state.metas.clone()))
                };
            }

            result.push(room_info);
        }

        Ok(result)
    }
}
