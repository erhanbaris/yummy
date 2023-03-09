use std::cell::RefCell;
use std::rc::Rc;

use rustpython_derive::{pyclass, PyPayload};
use rustpython_vm::builtins::{PyIntRef};
/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
use rustpython_vm::PyObjectRef;
use rustpython_vm::{PyResult, VirtualMachine};

use crate::auth::model::DeviceIdAuthRequest;

/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
macro_rules! wrapper {
    ($model: ident, $wrapper: ident) => {
        impl ModelWrapper for $wrapper {
            type Entity = $model;
            fn wrap(entity: Rc<RefCell<Self::Entity>>) -> Self {
                $wrapper::new(entity)
            }
        }
    };
}

/* *************************************************** STRUCTS **************************************************** */
#[pyclass(module = false, name = "DeviceIdAuth")]
#[derive(Debug, PyPayload)]
pub struct DeviceIdAuthRequestWrapper {
    data: Rc<RefCell<DeviceIdAuthRequest>>
}

#[pyclass(flags(BASETYPE))]
impl DeviceIdAuthRequestWrapper {
    pub fn new(data: Rc<RefCell<DeviceIdAuthRequest>>) -> Self {
        Self { data }
    }

    #[pymethod]
    pub fn get_request_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        match self.data.borrow().request_id {
            Some(request_id) => Ok(vm.ctx.new_float(request_id as f64).into()),
            None => Ok(vm.ctx.none().into())
        }
    }

    #[pymethod]
    pub fn set_request_id(&self, device_id: Option<PyIntRef>, _: &VirtualMachine) -> PyResult<()> {
        self.data.borrow_mut().request_id = device_id.map(|item| item.as_u32_mask() as usize);
        Ok(())
    }

    #[pymethod]
    pub fn get_device_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        Ok(vm.ctx.new_str(&self.data.borrow_mut().id[..]).into())
    }

    #[pymethod]
    pub fn set_device_id(&self, device_id: String, _: &VirtualMachine) -> PyResult<()> {
        self.data.borrow_mut().id = device_id;
        Ok(())
    }
}

/* **************************************************** ENUMS ***************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
pub trait ModelWrapper {
    type Entity;

    fn wrap(entity: Rc<RefCell<Self::Entity>>) -> Self;
}

/* ************************************************* IMPLEMENTS *************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* ************************************************* MACROS CALL ************************************************** */
wrapper!(DeviceIdAuthRequest, DeviceIdAuthRequestWrapper);

/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */