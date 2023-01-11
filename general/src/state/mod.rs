#[cfg(not(feature = "stateless"))]
pub mod inmemory;

#[cfg(feature = "stateless")]
pub mod stateless;

#[cfg(not(feature = "stateless"))]
pub use crate::state::inmemory::YummyState;

#[cfg(feature = "stateless")]
pub use crate::state::stateless::YummyState;

use std::collections::HashMap;
use std::borrow::Cow;
use std::sync::Arc;
use std::{fmt::Debug, borrow::Borrow};

use actix::Message;
use serde::de::DeserializeOwned;
use serde::ser::SerializeMap;
use strum_macros::EnumDiscriminants;
use serde::{Serialize, Deserialize, Serializer};
use thiserror::Error;

use crate::meta::{RoomMetaAccess, MetaType};
use crate::model::{UserId, RoomId};
use crate::model::CreateRoomAccessType;
use crate::model::RoomUserType;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum YummyStateError {
    #[error("Room not found")]
    RoomNotFound,

    #[error("User not found")]
    UserNotFound,
    
    #[error("User already in room")]
    UserAlreadInRoom,
    
    #[error("Already requested")]
    AlreadyRequested,
    
    #[error("User could not found in the room")]
    UserCouldNotFoundInRoom,
    
    #[error("Room has max users")]
    RoomHasMaxUsers,
    
    #[error("Cache could not readed")]
    CacheCouldNotReaded
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
    Description(Option<String>),
    Users(Vec<RoomUserInformation>),
    MaxUser(usize),
    UserLength(usize),
    AccessType(CreateRoomAccessType),
    Tags(Vec<String>),
    Metas(HashMap<String, MetaType<RoomMetaAccess>>),
    InsertDate(i32),
    JoinRequest(bool)
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RoomInfoTypeCollection {
    pub room_id: Option<RoomId>,
    pub items: Vec<RoomInfoType>
}

macro_rules! generate_room_type_getter {
    ($name: ident, $variant: path, $response: ty) => {
        pub fn $name(&self) -> Cow<'_, $response> {        
            for item in self.items.iter() {
                match item {
                    $variant(value) => return Cow::Borrowed(value),
                    _ => ()
                };
            }
    
            Cow::Owned(<$response>::default())
        }
    }
}

impl RoomInfoTypeCollection {       
    generate_room_type_getter!(get_room_name, RoomInfoType::RoomName, Option<String>);
    generate_room_type_getter!(get_description, RoomInfoType::Description, Option<String>);
    generate_room_type_getter!(get_users, RoomInfoType::Users, Vec<RoomUserInformation>);
    generate_room_type_getter!(get_max_user, RoomInfoType::MaxUser, usize);
    generate_room_type_getter!(get_user_length, RoomInfoType::UserLength, usize);
    generate_room_type_getter!(get_access_type, RoomInfoType::AccessType, CreateRoomAccessType);
    generate_room_type_getter!(get_tags, RoomInfoType::Tags, Vec<String>);
    generate_room_type_getter!(get_metas, RoomInfoType::Metas, HashMap<String, MetaType<RoomMetaAccess>>);
    generate_room_type_getter!(get_insert_date, RoomInfoType::InsertDate, i32);
    generate_room_type_getter!(get_join_request, RoomInfoType::JoinRequest, bool);

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
                RoomInfoType::Description(description) => items.serialize_entry("description", description),
                RoomInfoType::Users(users) => items.serialize_entry("users", users),
                RoomInfoType::MaxUser(max_user) => items.serialize_entry("max-user", max_user),
                RoomInfoType::UserLength(user_length) => items.serialize_entry("user-length", user_length),
                RoomInfoType::AccessType(access_type) => items.serialize_entry("access-type", access_type),
                RoomInfoType::Tags(tags) => items.serialize_entry("tags", tags),
                RoomInfoType::Metas(tags) => items.serialize_entry("metas", tags),
                RoomInfoType::InsertDate(insert_date) => items.serialize_entry("insert-date", insert_date),
                RoomInfoType::JoinRequest(join_request) => items.serialize_entry("join-request", join_request),
            }?;
        }
        
        items.end()
    }
}


#[cfg(test)]
mod tests {
    use std::ops::Deref;

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

        state.close_session(&user_id, &session_id);

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
        
        state.close_session(&UserId::new(), &SessionId::new());

        assert!(!state.is_session_online(&SessionId::new()));
        assert!(!state.is_user_online(&UserId::new()));

        Ok(())
    }
    
    #[actix::test]
    async fn room_tests() -> anyhow::Result<()> {
        configure_environment();
        let mut config = get_configuration().deref().clone();

        #[cfg(feature = "stateless")] {  
            use rand::Rng;     
            config.redis_prefix = format!("{}:", rand::thread_rng().gen::<usize>().to_string());
        }
    
        let config = Arc::new(config);
        
        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();


        DummyActor{}.start().recipient::<SendMessage>();
        let mut state = YummyState::new(config, #[cfg(feature = "stateless")] conn);
        
        let room_1 = RoomId::new();
        state.create_room(&room_1, 1234, Some("room".to_string()), None, CreateRoomAccessType::Friend, 2, vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()], Some(HashMap::from([
            ("gender".to_string(), MetaType::String("Male".to_string(), RoomMetaAccess::User)),
            ("location".to_string(), MetaType::String("Copenhagen".to_string(), RoomMetaAccess::User)),
            ("postcode".to_string(), MetaType::Number(1000.0, RoomMetaAccess::Moderator)),
            ("score".to_string(), MetaType::Number(15.3, RoomMetaAccess::Anonymous)),
            ("temp_admin".to_string(), MetaType::Bool(true, RoomMetaAccess::Admin)),
        ])), false);

        let user_1 = UserId::new();
        let user_2 = UserId::new();
        let user_3 = UserId::new();

        let user_1_session = state.new_session(&user_1, None, UserType::User);
        let user_2_session = state.new_session(&user_2, None, UserType::User);
        let user_3_session = state.new_session(&user_3, None, UserType::User);
        
        state.join_to_room(&room_1, &user_1, &user_1_session, RoomUserType::Owner)?;
        assert_eq!(state.get_users_room_type(&user_1, &room_1).unwrap(), RoomUserType::Owner);

        assert_eq!(state.join_to_room(&room_1, &user_1, &user_1_session, RoomUserType::Owner).err().unwrap(), YummyStateError::UserAlreadInRoom);

        state.join_to_room(&room_1, &user_2, &user_2_session, RoomUserType::User)?;
        assert_eq!(state.get_users_room_type(&user_2, &room_1).unwrap(), RoomUserType::User);

        assert_eq!(state.join_to_room(&room_1, &user_3, &user_3_session, RoomUserType::Owner).err().unwrap(), YummyStateError::RoomHasMaxUsers);
        assert_eq!(state.join_to_room(&room_1, &user_2, &user_2_session, RoomUserType::Owner).err().unwrap(), YummyStateError::RoomHasMaxUsers);

        assert_eq!(state.join_to_room(&RoomId::new(), &UserId::new(), &SessionId::new(), RoomUserType::Owner).err().unwrap(), YummyStateError::RoomNotFound);
        assert_eq!(state.get_users_from_room(&room_1)?.len(), 2);

        assert_eq!(state.disconnect_from_room(&room_1, &user_1, &user_1_session)?, false);
        assert_eq!(state.get_users_from_room(&room_1)?.len(), 1);

        assert_eq!(state.disconnect_from_room(&room_1, &user_2, &user_2_session)?, true);
        assert!(state.get_users_from_room(&room_1).is_err());

        assert!(!state.is_empty());

        state.close_session(&user_1, &user_1_session);
        state.close_session(&user_2, &user_2_session);
        state.close_session(&user_3, &user_3_session);

        assert!(state.is_empty());

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
        state.create_room(&room, 1234, None, None, CreateRoomAccessType::Public, 0, Vec::new(), None, false);

        for _ in 0..100_000 {
            let user_id = UserId::new();
            let session_id = SessionId::new();
            state.new_session(&user_id, None, UserType::User);
            state.join_to_room(&room, &user_id, &session_id, RoomUserType::Owner)?
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
        state.create_room(&room, 1234, Some("Room 1".to_string()), None, CreateRoomAccessType::Private, 10, vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()], None, false);

        let result = state.get_room_info(&room, RoomMetaAccess::Admin, Vec::new())?;
        assert_eq!(result.items.len(), 0);

        let result = state.get_room_info(&room, RoomMetaAccess::Admin, vec![RoomInfoTypeVariant::RoomName])?;
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.get_room_name().into_owned(), Some("Room 1".to_string()));

        state.set_room_info(&room, vec![RoomInfoType::RoomName(Some("New room".to_string()))]);

        let result = state.get_room_info(&room, RoomMetaAccess::Admin, vec![RoomInfoTypeVariant::RoomName])?;
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.get_room_name().into_owned(), Some("New room".to_string()));

        let result = state.get_room_info(&room, RoomMetaAccess::Admin, vec![RoomInfoTypeVariant::Tags, RoomInfoTypeVariant::InsertDate, RoomInfoTypeVariant::RoomName, RoomInfoTypeVariant::AccessType, RoomInfoTypeVariant::Users, RoomInfoTypeVariant::MaxUser, RoomInfoTypeVariant::UserLength])?;
        assert_eq!(result.items.len(), 7);
        assert_eq!(result.get_room_name().into_owned(), Some("New room".to_string()));
        assert_eq!(result.get_max_user().into_owned(), 10);
        assert_eq!(result.get_user_length().into_owned(), 0);
        assert_eq!(result.get_access_type().into_owned(), CreateRoomAccessType::Private);
        assert!(result.get_tags().len() > 0);
        assert!(result.get_insert_date().into_owned() > 0);

        // Tag update test
        let mut tags: Vec<String> = result.get_tags().into_owned();
        tags.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(tags, vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()]);

        state.set_room_info(&room, vec![RoomInfoType::Tags(vec!["yummy1".to_string(), "yummy2".to_string(), "yummy3".to_string()])]);
        let result = state.get_room_info(&room, RoomMetaAccess::Admin, vec![RoomInfoTypeVariant::Tags])?;
        
        let mut tags = result.get_tags().into_owned();
        tags.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(tags, vec!["yummy1".to_string(), "yummy2".to_string(), "yummy3".to_string()]);

        state.set_room_info(&room, vec![RoomInfoType::Tags(Vec::new())]);
        let result = state.get_room_info(&room, RoomMetaAccess::Admin, vec![RoomInfoTypeVariant::Tags])?;
        
        let tags = result.get_tags().into_owned();
        assert_eq!(tags, Vec::<String>::new());

        let user_1 = UserId::new();
        let user_2 = UserId::new();
        let user_3 = UserId::new();

        let user_1_session = SessionId::new();
        let user_2_session = SessionId::new();
        let user_3_session = SessionId::new();

        state.new_session(&user_1, Some("user1".to_string()), UserType::User);
        assert_eq!(state.get_user_type(&user_1), Some(UserType::User));

        state.new_session(&user_2, Some("user2".to_string()), UserType::Mod);
        assert_eq!(state.get_user_type(&user_2), Some(UserType::Mod));

        state.new_session(&user_3, Some("user3".to_string()), UserType::Admin);
        assert_eq!(state.get_user_type(&user_3), Some(UserType::Admin));

        state.join_to_room(&room, &user_1, &user_1_session, RoomUserType::Owner)?;
        state.join_to_room(&room, &user_2, &user_2_session, RoomUserType::Owner)?;
        state.join_to_room(&room, &user_3, &user_3_session, RoomUserType::Owner)?;
        
        let result = state.get_room_info(&room, RoomMetaAccess::Admin, vec![RoomInfoTypeVariant::UserLength, RoomInfoTypeVariant::Users])?;
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.get_user_length().into_owned(), 3);

        let mut users: Vec<RoomUserInformation> = result.get_users().into_owned();
        users.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
        assert_eq!(users, vec![RoomUserInformation { user_id: Arc::new(user_1.clone()), name: Some("user1".to_string()), user_type: RoomUserType::Owner }, RoomUserInformation { user_id: Arc::new(user_2.clone()), name: Some("user2".to_string()), user_type: RoomUserType::Owner }, RoomUserInformation { user_id: Arc::new(user_3.clone()), name: Some("user3".to_string()), user_type: RoomUserType::Owner }]);

        // Change user permission
        state.set_users_room_type(&user_1, &room, RoomUserType::User);
        
        let result = state.get_room_info(&room, RoomMetaAccess::Admin, vec![RoomInfoTypeVariant::Users])?;
        assert_eq!(result.items.len(), 1);

        let mut users: Vec<RoomUserInformation> = result.get_users().into_owned();
        users.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
        assert_eq!(users, vec![RoomUserInformation { user_id: Arc::new(user_1), name: Some("user1".to_string()), user_type: RoomUserType::User }, RoomUserInformation { user_id: Arc::new(user_2), name: Some("user2".to_string()), user_type: RoomUserType::Owner }, RoomUserInformation { user_id: Arc::new(user_3), name: Some("user3".to_string()), user_type: RoomUserType::Owner }]);
        
        Ok(())
    }
    
    macro_rules! meta_validation {
        ($state: expr, $room_id: expr, $access: expr, $len: expr, $map: expr) => {
            let metas = $state.get_room_info(&$room_id, $access, vec![RoomInfoTypeVariant::Metas])?;
            let item = metas.get_metas().into_owned();
    
            assert_eq!(item.len(), $len);
            assert_eq!(item, $map);
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
        state.create_room(&room_id, 1234, Some("room".to_string()), None, CreateRoomAccessType::Friend, 2, vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()], Some(HashMap::from([
            ("gender".to_string(), MetaType::String("Male".to_string(), RoomMetaAccess::User)),
            ("location".to_string(), MetaType::String("Copenhagen".to_string(), RoomMetaAccess::User)),
            ("postcode".to_string(), MetaType::Number(1000.0, RoomMetaAccess::Moderator)),
            ("score".to_string(), MetaType::Number(15.3, RoomMetaAccess::Anonymous)),
            ("temp_admin".to_string(), MetaType::Bool(true, RoomMetaAccess::Admin)),
        ])), false);

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

    #[actix::test]
    async fn room_meta_update_test() -> anyhow::Result<()> {
        configure_environment();
        let config = get_configuration();
        
        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();

        DummyActor{}.start().recipient::<SendMessage>();
        let mut state = YummyState::new(config, #[cfg(feature = "stateless")] conn);
        
        let room_id = RoomId::new();
        state.create_room(&room_id, 1234, Some("room".to_string()), None, CreateRoomAccessType::Friend, 2, vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()], Some(HashMap::from([
            ("gender".to_string(), MetaType::String("Male".to_string(), RoomMetaAccess::User)),
            ("location".to_string(), MetaType::String("Copenhagen".to_string(), RoomMetaAccess::User)),
            ("postcode".to_string(), MetaType::Number(1000.0, RoomMetaAccess::Moderator)),
            ("score".to_string(), MetaType::Number(15.3, RoomMetaAccess::Anonymous)),
            ("temp_admin".to_string(), MetaType::Bool(true, RoomMetaAccess::Admin)),
        ])), false);

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

        // Update room
        state.set_room_info(&room_id, vec![RoomInfoType::Metas(HashMap::from([
            ("gender".to_string(), MetaType::String("Female".to_string(), RoomMetaAccess::User)),
            ("location".to_string(), MetaType::String("Oslo".to_string(), RoomMetaAccess::User)),
            ("postcode".to_string(), MetaType::Number(2000.0, RoomMetaAccess::Moderator)),
            ("score".to_string(), MetaType::Number(30.0, RoomMetaAccess::Anonymous)),
            ("test".to_string(), MetaType::Number(1.0, RoomMetaAccess::Anonymous)),
        ]))]);


        meta_validation!(state, room_id, RoomMetaAccess::Anonymous, 2, HashMap::from([
            ("score".to_string(), MetaType::Number(30.0, RoomMetaAccess::Anonymous)),
            ("test".to_string(), MetaType::Number(1.0, RoomMetaAccess::Anonymous))
        ]));

        meta_validation!(state, room_id, RoomMetaAccess::User, 4, HashMap::from([
            ("gender".to_string(), MetaType::String("Female".to_string(), RoomMetaAccess::User)),
            ("location".to_string(), MetaType::String("Oslo".to_string(), RoomMetaAccess::User)),
            ("score".to_string(), MetaType::Number(30.0, RoomMetaAccess::Anonymous)),
            ("test".to_string(), MetaType::Number(1.0, RoomMetaAccess::Anonymous))
        ]));

        meta_validation!(state, room_id, RoomMetaAccess::Moderator, 5, HashMap::from([
            ("gender".to_string(), MetaType::String("Female".to_string(), RoomMetaAccess::User)),
            ("location".to_string(), MetaType::String("Oslo".to_string(), RoomMetaAccess::User)),
            ("postcode".to_string(), MetaType::Number(2000.0, RoomMetaAccess::Moderator)),
            ("score".to_string(), MetaType::Number(30.0, RoomMetaAccess::Anonymous)),
            ("test".to_string(), MetaType::Number(1.0, RoomMetaAccess::Anonymous))
        ]));

        meta_validation!(state, room_id, RoomMetaAccess::Admin, 5, HashMap::from([
            ("gender".to_string(), MetaType::String("Female".to_string(), RoomMetaAccess::User)),
            ("location".to_string(), MetaType::String("Oslo".to_string(), RoomMetaAccess::User)),
            ("postcode".to_string(), MetaType::Number(2000.0, RoomMetaAccess::Moderator)),
            ("score".to_string(), MetaType::Number(30.0, RoomMetaAccess::Anonymous)),
            ("test".to_string(), MetaType::Number(1.0, RoomMetaAccess::Anonymous))
        ]));

        Ok(())
    }

    #[actix::test]
    async fn join_request_test() -> anyhow::Result<()> {
        configure_environment();
        let mut config = get_configuration().deref().clone();

        #[cfg(feature = "stateless")] {  
            use rand::Rng;     
            config.redis_prefix = format!("{}:", rand::thread_rng().gen::<usize>().to_string());
        }
    
        let config = Arc::new(config);
        
        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();


        DummyActor{}.start().recipient::<SendMessage>();
        let mut state = YummyState::new(config, #[cfg(feature = "stateless")] conn);
        
        let room_id = RoomId::new();
        state.create_room(&room_id, 1234, Some("room".to_string()), None, CreateRoomAccessType::Friend, 2, vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()], Some(HashMap::from([
            ("gender".to_string(), MetaType::String("Male".to_string(), RoomMetaAccess::User)),
            ("location".to_string(), MetaType::String("Copenhagen".to_string(), RoomMetaAccess::User)),
            ("postcode".to_string(), MetaType::Number(1000.0, RoomMetaAccess::Moderator)),
            ("score".to_string(), MetaType::Number(15.3, RoomMetaAccess::Anonymous)),
            ("temp_admin".to_string(), MetaType::Bool(true, RoomMetaAccess::Admin)),
        ])), true);

        let user_1 = UserId::new();
        let user_2 = UserId::new();
        let user_3 = UserId::new();
        let user_4 = UserId::new();

        let user_1_session = state.new_session(&user_1, None, UserType::User);
        let user_2_session = state.new_session(&user_2, None, UserType::User);
        let user_3_session = state.new_session(&user_3, None, UserType::User);
        let user_4_session = state.new_session(&user_4, None, UserType::User);
        
        state.join_to_room(&room_id, &user_1, &user_1_session, RoomUserType::Owner)?;
        state.join_to_room_request(&room_id, &user_2, &user_2_session, RoomUserType::User)?;
        state.join_to_room_request(&room_id, &user_3, &user_3_session, RoomUserType::Moderator)?;
        state.join_to_room_request(&room_id, &user_4, &user_4_session, RoomUserType::Owner)?;

        let waiting_users = state.get_join_requests(&room_id)?;
        assert_eq!(waiting_users.len(), 3);

        assert_eq!(waiting_users.get(&user_2).cloned(), Some(RoomUserType::User));
        assert_eq!(waiting_users.get(&user_3).cloned(), Some(RoomUserType::Moderator));
        assert_eq!(waiting_users.get(&user_4).cloned(), Some(RoomUserType::Owner));

        assert_eq!(state.disconnect_from_room(&room_id, &user_1, &user_1_session)?, true);

        assert!(!state.is_empty());

        state.close_session(&user_1, &user_1_session);
        state.close_session(&user_2, &user_2_session);
        state.close_session(&user_3, &user_3_session);
        state.close_session(&user_4, &user_4_session);

        assert!(state.is_empty());

        Ok(())
    }
    
}
