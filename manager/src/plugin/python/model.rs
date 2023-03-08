use std::sync::Arc;

use rustpython_vm::builtins::PyTypeRef;
use rustpython_vm::common::static_cell;
use rustpython_vm::types::Constructor;
/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
use rustpython_vm::{convert::ToPyObject, object::PyWeak, class::PyClassImpl};
use rustpython_vm::{builtins::PyList, PyObjectRef};
use rustpython_vm::{
    pyclass, pymodule, PyObject, PyPayload, PyResult, TryFromBorrowedObject, VirtualMachine,
};
use testing::client::DummyClient;

/*
pub struct DeviceIdAuthRequestWrapper(pub DeviceIdAuthRequest);
impl ToPyObject for DeviceIdAuthRequestWrapper {
    fn to_pyobject(self, vm: &VirtualMachine) -> PyObjectRef {
        let mut class = vm.ctx.new_class(
            None,
            "DeviceIdAuth",
            vm.ctx.types.weakref_type.to_owned(),
            PyWeak::make_slots(),
        );

        //class.set_attr(vm.ctx.new_str("device_id"), vm.ctx.new_method("flush", cls, |_self: PyObjectRef| {}));

        class.into()
    }
}
*/

use crate::auth::model::DeviceIdAuthRequest;

/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* *************************************************** STRUCTS **************************************************** */
#[pyclass(module = false, name = "DeviceIdAuth")]
#[derive(Debug, PyPayload)]
pub struct DeviceIdAuthRequestWrapper(pub DeviceIdAuthRequest);


#[pyclass(flags(BASETYPE))]
impl DeviceIdAuthRequestWrapper {
    

    #[pymethod(magic)]
    pub fn device_id(&self) -> PyResult<String> {
        Ok(self.0.id.clone())
    }
}

/* **************************************************** ENUMS ***************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */


/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */