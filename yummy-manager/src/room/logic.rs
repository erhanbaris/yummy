/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::{marker::PhantomData, sync::Arc};

use yummy_cache::state::YummyState;
use yummy_database::DatabaseTrait;
use yummy_model::meta::collection::RoomMetaCollectionInformation;
use yummy_model::RoomId;
use yummy_model::config::YummyConfig;
use yummy_model::meta::{RoomMetaType, RoomMetaAccess};
use yummy_general::database::Pool;

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
#[derive(Clone)]
pub struct RoomLogic<DB: DatabaseTrait + ?Sized> {
    config: Arc<YummyConfig>,
    database: Arc<Pool>,
    states: YummyState,
    _marker: PhantomData<DB>
}

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* **************************************************************************************************************** */
impl<DB: DatabaseTrait + ?Sized> RoomLogic<DB> {
    pub fn new(config: Arc<YummyConfig>, states: YummyState, database: Arc<Pool>) -> Self {
        Self {
            config,
            database,
            states,
            _marker: PhantomData
        }
    }

    pub fn get_room_meta(&self, room_id: RoomId, key: String) -> anyhow::Result<Option<RoomMetaType>> {
        Ok(self.states.get_room_meta(&room_id, RoomMetaAccess::System)?
            .get_with_name(&key)
            .cloned()
            .map(|item| item.meta))
    }

    pub fn get_room_metas(&self, room_id: RoomId) -> anyhow::Result<Vec<RoomMetaCollectionInformation>> {
        Ok(self.states.get_room_metas(&room_id)?)
    }

    pub fn set_room_meta(&self, room_id: RoomId, key: String, value: RoomMetaType) -> anyhow::Result<()> {
        Ok(self.states.set_room_meta(&room_id, key, value)?)
    }

    pub fn remove_all_metas(&self, room_id: RoomId) -> anyhow::Result<()> {
        Ok(self.states.remove_all_room_metas(&room_id)?)
    }

    pub fn remove_room_meta(&self, room_id: RoomId, key: String) -> anyhow::Result<()> {
        Ok(self.states.remove_room_meta(&room_id, key)?)
    }
}

/* **************************************************************************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */
