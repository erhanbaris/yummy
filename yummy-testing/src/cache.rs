/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use yummy_cache::{state::resource::YummyCacheResourceFactory, cache::YummyCacheResource};
use yummy_model::{UserId, UserType, meta::collection::{UserMetaCollection, RoomMetaCollection}, UserInformationModel, RoomId};

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
pub struct DummyResourceFactory;
pub struct DummyUserInformationResource;
pub struct DummyUserMetaResource;
pub struct DummyUserTypeResource;
pub struct DummyRoomMetaResource;

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* **************************************************************************************************************** */
impl YummyCacheResourceFactory for DummyResourceFactory {
    fn user_information(&self) -> Box<dyn YummyCacheResource<K=UserId, V=UserInformationModel>> {
        Box::new(DummyUserInformationResource {})
    }

    fn user_metas(&self) -> Box<dyn YummyCacheResource<K=UserId, V=UserMetaCollection>> {
        Box::new(DummyUserMetaResource {})
    }

    fn user_type(&self) -> Box<dyn YummyCacheResource<K=UserId, V=UserType>> {
        Box::new(DummyUserTypeResource {})
    }

    fn room_metas(&self) -> Box<dyn YummyCacheResource<K=yummy_model::RoomId, V=yummy_model::meta::collection::RoomMetaCollection>> {
        Box::new(DummyRoomMetaResource {})
    }
}

impl YummyCacheResource for DummyUserInformationResource {
    type K=UserId;
    type V=UserInformationModel;

    fn get(&self, _: &Self::K) -> anyhow::Result<Option<Self::V>> { Ok(None) }
}

impl YummyCacheResource for DummyUserMetaResource {
    type K=UserId;
    type V=UserMetaCollection;

    fn get(&self, _: &Self::K) -> anyhow::Result<Option<Self::V>> { Ok(None) }
}

impl YummyCacheResource for DummyUserTypeResource {
    type K=UserId;
    type V=UserType;

    fn get(&self, _: &Self::K) -> anyhow::Result<Option<Self::V>> { Ok(None) }
}

impl YummyCacheResource for DummyRoomMetaResource {
    type K=RoomId;
    type V=RoomMetaCollection;

    fn get(&self, _: &Self::K) -> anyhow::Result<Option<Self::V>> { Ok(None) }
}

/* **************************************************************************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */
