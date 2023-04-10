/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use yummy_model::{UserId, UserInformationModel, meta::collection::{UserMetaCollection, RoomMetaCollection}, UserType, RoomId};
use crate::cache::YummyCacheResource;

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* **************************************************************************************************************** */
pub trait YummyCacheResourceFactory {
    fn user_information(&self) -> Box<dyn YummyCacheResource<K=UserId, V=UserInformationModel>>;
    fn user_metas(&self) -> Box<dyn YummyCacheResource<K=UserId, V=UserMetaCollection>>;
    fn user_type(&self) -> Box<dyn YummyCacheResource<K=UserId, V=UserType>>;
    fn room_metas(&self) -> Box<dyn YummyCacheResource<K=RoomId, V=RoomMetaCollection>>;
}

/* **************************************************************************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */

