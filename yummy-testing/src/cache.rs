/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use yummy_cache::{state::resource::YummyCacheResourceFactory, cache::YummyCacheResource};
use yummy_model::{UserId, UserType, meta::collection::UserMetaCollection, UserInformationModel};

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
pub struct DummyResourceFactory;
pub struct DummyUserInformationResource;
pub struct DummyUserMetaResource;
pub struct DummyUserTypeResource;

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

/* **************************************************************************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */
