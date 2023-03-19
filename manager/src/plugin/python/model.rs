/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use database::DefaultDatabaseStore;
use rustpython::vm::{PyPayload, pyclass};
use crate::plugin::YummyPluginContext;

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */

#[pyclass(module = false, name = "YummyPluginContext")]
#[derive(PyPayload)]
pub struct YummyPluginContextWrapper {
    pub data: YummyPluginContext<DefaultDatabaseStore>
}

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* **************************************************************************************************************** */
pub trait ModelWrapper {
    type Entity;
    fn wrap(entity: Rc<RefCell<Self::Entity>>) -> Self;
}

/* **************************************************************************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* **************************************************************************************************************** */
#[pyclass(flags(BASETYPE))]
impl YummyPluginContextWrapper {
    pub fn new(data: YummyPluginContext<DefaultDatabaseStore>) -> Self {
        Self { data }
    }
}

/* **************************************************************************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* **************************************************************************************************************** */
impl Debug for YummyPluginContextWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("YummyPluginContextWrapper").finish()
    }
}

unsafe impl Send for YummyPluginContextWrapper {}
unsafe impl Sync for YummyPluginContextWrapper {}

/* **************************************************************************************************************** */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */