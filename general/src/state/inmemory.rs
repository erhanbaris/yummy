use std::collections::HashMap;
use std::sync::Arc;
use std::borrow::Borrow;

use crate::config::YummyConfig;
use crate::meta::{RoomMetaAccess, MetaType};
use crate::model::{UserId, RoomId, SessionId};
use crate::model::CreateRoomAccessType;
use crate::model::RoomUserType;
use crate::model::UserType;

use super::*;

#[derive(Clone)]
pub struct YummyState {
    #[allow(dead_code)]
    config: Arc<YummyConfig>,
    user: Arc<parking_lot::Mutex<std::collections::HashMap<UserId, crate::model::UserState>>>,
    room: Arc<parking_lot::Mutex<std::collections::HashMap<RoomId, crate::model::RoomState>>>,
    session_to_user: Arc<parking_lot::Mutex<std::collections::HashMap<SessionId, UserId>>>,
    session_to_room: Arc<parking_lot::Mutex<std::collections::HashMap<SessionId, std::collections::HashSet<RoomId>>>>,
}

impl YummyState {
    pub fn new(config: Arc<YummyConfig>) -> Self {
        Self {
            config,

            user: Arc::new(parking_lot::Mutex::default()),
            room: Arc::new(parking_lot::Mutex::default()),
            session_to_user: Arc::new(parking_lot::Mutex::default()),
            session_to_room: Arc::new(parking_lot::Mutex::default())            
        }
    }
}

impl YummyState {
    
    #[tracing::instrument(name="is_empty", skip(self))]
    pub fn is_empty(&self) -> bool {
        self.room.lock().len() == 0 &&
            self.user.lock().len() == 0 &&
            self.session_to_room.lock().len() == 0 &&
            self.session_to_user.lock().len() == 0
    }

    #[tracing::instrument(name="is_user_online", skip(self))]
    pub fn is_user_online(&self, user_id: &UserId) -> bool {
        self.user.lock().contains_key(user_id)
    }

    #[tracing::instrument(name="get_user_type", skip(self))]
    pub fn get_user_type(&mut self, user_id: &UserId) -> Option<UserType> {
        self.user.lock().get(user_id).map(|user| user.user_type)
    }
    
    
    #[tracing::instrument(name="get_users_room_type", skip(self))]
    pub fn get_users_room_type(&mut self, user_id: &UserId, room_id: &RoomId) -> Option<RoomUserType> {
        match self.room.lock().get(room_id) {
            Some(room) => room.users.lock().get(user_id).cloned(),
            None => None
        }
    }

    #[tracing::instrument(name="set_users_room_type", skip(self))]
    pub fn set_users_room_type(&mut self, user_id: &UserId, room_id: &RoomId, user_type: RoomUserType) {
        if let Some(room) = self.room.lock().get_mut(room_id.borrow()) {
            match room.users.lock().get_mut(user_id) {
                Some(user) => *user = user_type,
                None => ()
            };
        }
    }

    #[tracing::instrument(name="is_session_online", skip(self))]
    pub fn is_session_online<T: Borrow<SessionId> + std::fmt::Debug>(&self, session_id: T) -> bool {
        self.session_to_user.lock().contains_key(session_id.borrow())
    }

    #[tracing::instrument(name="new_session", skip(self))]
    pub fn new_session(&self, user_id: &UserId, name: Option<String>, user_type: UserType) -> SessionId {
        use std::collections::HashSet;

        let session_id = SessionId::new();
        self.session_to_user.lock().insert(session_id.clone(), user_id.clone());

        let mut users = self.user.lock();

        match users.get_mut(&user_id) {
            Some(user) => {
                user.sessions.insert(session_id.clone());
            },
            None => {
                users.insert(user_id.clone(), crate::model::UserState { user_id: user_id.clone(), name, sessions: HashSet::from([session_id.clone()]), user_type, join_requests: HashMap::default() });
            }
        }
        session_id
    }

    #[tracing::instrument(name="set_user_type", skip(self))]
    pub fn set_user_type(&self, user_id: &UserId, user_type: UserType) {
        if let Some(user) = self.user.lock().get_mut(user_id) {
            user.user_type = user_type
        }
    }

    #[tracing::instrument(name="set_user_name", skip(self))]
    pub fn set_user_name(&self, user_id: &UserId, name: String) {
        if let Some(user) = self.user.lock().get_mut(user_id) {
            user.name = Some(name)
        }
    }

    #[tracing::instrument(name="close_session", skip(self))]
    pub fn close_session(&self, user_id: &UserId, session_id: &SessionId) -> bool {
        let user_id = self.session_to_user.lock().remove(session_id);

        match user_id {
            Some(user_id) => {
                let session_count = match self.user.lock().get_mut(&user_id) {
                    Some(user) => {
                        user.sessions.remove(session_id);
                        user.sessions.len()
                    }
                    None => 0
                };

                if session_count == 0 {
                    self.user.lock().remove(&user_id);
                }
                
                true
            },
            None => false
        }
    }

    #[tracing::instrument(name="get_user_rooms", skip(self))]
    pub fn get_user_rooms(&self, user_id: &UserId, session_id: &SessionId) -> Option<Vec<RoomId>> {
        self.session_to_room.lock().get(session_id).map(|rooms| rooms.iter().cloned().collect::<Vec<_>>())
    }

    #[tracing::instrument(name="create_room", skip(self))]
    pub fn create_room(&self, room_id: &RoomId, insert_date: i32, name: Option<String>, description: Option<String>, access_type: CreateRoomAccessType, max_user: usize, tags: Vec<String>, metas: Option<HashMap<String, MetaType<RoomMetaAccess>>>, join_request: bool) {
        use std::collections::HashMap;

        self.room.lock().insert(*room_id, crate::model::RoomState {
            max_user,
            room_id: *room_id,
            insert_date,
            users: parking_lot::Mutex::new(HashMap::new()),
            tags,
            name,
            description,
            access_type,
            metas: metas.unwrap_or_default(),
            join_request,
            join_requests: parking_lot::Mutex::default()
        });
    }

    #[tracing::instrument(name="join_to_room_request", skip(self))]
    pub fn join_to_room_request(&mut self, room_id: &RoomId, user_id: &UserId, session_id: &SessionId, user_type: crate::model::RoomUserType) -> Result<(), YummyStateError> {
        match self.room.lock().get_mut(room_id.borrow()) {
            Some(room) => {

                if room.users.lock().contains_key(user_id) {
                    return Err(YummyStateError::UserAlreadInRoom);
                }
                
                let users_len = room.users.lock().len();

                // If the max_user 0 or lower than users count, add to room
                if room.max_user == 0 || room.max_user > users_len {
                    let inserted = room.join_requests.lock().insert(user_id.clone(), user_type);

                    if inserted.is_some() {
                        return Err(YummyStateError::AlreadyRequested)
                    }

                    match self.user.lock().get_mut(&user_id) {
                        Some(user) => user.join_requests.insert(room_id.clone(), session_id.clone()),
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
        match self.room.lock().get_mut(room_id.borrow()) {
            Some(room) => {
                let room_user_type = match room.join_requests.lock().remove(user_id) {
                    Some(room_user_type) => room_user_type,
                    None => return Err(YummyStateError::UserNotFound)
                };

                let session_id = match self.user.lock().get_mut(user_id) {
                    Some(user) => user.join_requests.remove(room_id),
                    None => return Err(YummyStateError::UserNotFound)
                };

                let session_id = match session_id {
                    Some(session_id) => session_id,
                    None => return Err(YummyStateError::UserNotFound)
                };

                Ok((session_id, room_user_type))
            },
            None => Err(YummyStateError::RoomNotFound)
        }
    }

    #[tracing::instrument(name="join_to_room", skip(self))]
    pub fn join_to_room(&mut self, room_id: &RoomId, user_id: &UserId, session_id: &SessionId, user_type: crate::model::RoomUserType) -> Result<(), YummyStateError> {
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
                    
                    let mut session_to_room =  self.session_to_room.lock();
                    match session_to_room.get_mut(session_id) {
                        Some(session_rooms) => {
                            session_rooms.insert(room_id.clone());
                        },
                        None => {
                            session_to_room.insert(session_id.clone(), std::collections::HashSet::from([room_id.clone()]));
                        }
                    };
                    Ok(())
                } else {
                    Err(YummyStateError::RoomHasMaxUsers)
                }
            }
            None => Err(YummyStateError::RoomNotFound)
        }
    }

    #[tracing::instrument(name="join_to_room", skip(self))]
    pub fn disconnect_from_room(&self, room_id: &RoomId, user_id: &UserId, session: &SessionId) -> Result<bool, YummyStateError> {
        let mut rooms = self.room.lock();
        let room_removed = match rooms.get_mut(room_id.borrow()) {
            Some(room) => {
                let mut users = room.users.lock();
                let mut session_to_room = self.session_to_room.lock();
                
                let room_count = match session_to_room.get_mut(session) {
                    Some(rooms) => {
                        rooms.remove(room_id);
                        rooms.len()
                    },
                    None => 0
                };

                if room_count == 0 {
                    session_to_room.remove(session);
                }

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

    #[tracing::instrument(name="get_users_from_room", skip(self))]
    pub fn get_users_from_room(&self, room_id: &RoomId) -> Result<Vec<Arc<UserId>>, YummyStateError> {
        match self.room.lock().get_mut(room_id) {
            Some(room) => Ok(room.users.lock().keys().map(|item| Arc::new(item.clone())).collect::<Vec<_>>()), // todo: discart cloning
            None => Err(YummyStateError::RoomNotFound)
        }
    }

    #[tracing::instrument(name="get_user_location", skip(self))]
    pub fn get_user_location(&self, user_id: &UserId) -> Option<String> {
        None
    }

    #[tracing::instrument(name="get_join_requests", skip(self))]
    pub fn get_join_requests(&self, room_id: &RoomId) -> Result<HashMap<UserId, RoomUserType>, YummyStateError> {
        match self.room.lock().get(room_id) {
            Some(room) => Ok(room.join_requests.lock().clone()),
            None => Err(YummyStateError::RoomNotFound)
        }
    }

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
                        RoomInfoTypeVariant::Description => RoomInfoType::Description(room.description.clone()),
                        RoomInfoTypeVariant::UserLength => RoomInfoType::UserLength(room.users.lock().len()),
                        RoomInfoTypeVariant::AccessType => RoomInfoType::AccessType(room.access_type.clone()),
                        RoomInfoTypeVariant::JoinRequest => RoomInfoType::JoinRequest(room.join_request),
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

    #[tracing::instrument(name="set_room_info", skip(self))]
    pub fn set_room_info(&self, room_id: &RoomId, query: Vec<RoomInfoType>) {
        match self.room.lock().get_mut(room_id) {
            Some(room) => {
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
                        RoomInfoType::InsertDate(_) => (),
                    };
                }
            },
            None => ()
        }
    }

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
                    RoomInfoTypeVariant::JoinRequest => room_info.items.push(RoomInfoType::JoinRequest(room_state.join_request)),
                    RoomInfoTypeVariant::RoomName => room_info.items.push(RoomInfoType::RoomName(room_state.name.clone())),
                    RoomInfoTypeVariant::Description => room_info.items.push(RoomInfoType::Description(room_state.description.clone())),
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
