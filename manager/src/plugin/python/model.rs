use std::borrow::Borrow;
use std::ops::DerefMut;
use std::sync::Arc;

use rustpython_derive::{pyclass, PyPayload};
/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
use rustpython_vm::builtins::{PyStrRef, PyStr};
use rustpython_vm::common::lock::PyRwLock;
use rustpython_vm::{convert::ToPyObject};
use rustpython_vm::{PyObjectRef, TryFromBorrowedObject};
use rustpython_vm::{PyResult, VirtualMachine, PyRef, py_class, extend_class,
};

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
pub struct DeviceIdAuthRequestWrapper {
    data: Arc<PyRwLock<DeviceIdAuthRequest>>
}

impl Drop for DeviceIdAuthRequestWrapper {
    fn drop(&mut self) {
        println!("Drop DeviceIdAuthRequestWrapper");
    }
}

impl TryFromBorrowedObject for DeviceIdAuthRequestWrapper {
    fn try_from_borrowed_object(vm: &VirtualMachine, obj: &rustpython_vm::PyObject) -> PyResult<Self> {
        obj.try_to_value(vm)
    }
}

#[pyclass(flags(BASETYPE))]
impl DeviceIdAuthRequestWrapper {
    pub fn new(data: Arc<PyRwLock<DeviceIdAuthRequest>>) -> Self {
        Self { data }
    }

    #[pymethod]
    pub fn get_device_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        Ok(vm.ctx.new_str(&self.data.read().id[..]).into())
    }

    #[pymethod]
    pub fn set_device_id(&self, device_id: String, _: &VirtualMachine) -> PyResult<()> {
        self.data.write().id = device_id;
        Ok(())
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