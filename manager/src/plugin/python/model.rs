/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::cell::RefCell;
use std::rc::Rc;

use rustpython_derive::{pyclass, PyPayload};
use rustpython_vm::builtins::{PyIntRef};
use rustpython_vm::PyObjectRef;
use rustpython_vm::{PyResult, VirtualMachine};

use crate::auth::model::DeviceIdAuthRequest;

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* **************************************************** MACROS **************************************************** */
/* **************************************************************************************************************** */
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

macro_rules! get_string {
    ($self: expr, $item: ident, $vm: ident) => {
        Ok($vm.ctx.new_str(&$self.data.borrow_mut().$item[..]).into())
    };
}

macro_rules! set_string {
    ($self: expr,  $target: ident, $source: ident) => {
        $self.data.borrow_mut().$target = $source;
    };
}

macro_rules! set_nullable_usize {
    ($self: expr, $target: ident, $number: ident) => {
        $self.data.borrow_mut().$target = $number.map(|item| item.as_u32_mask() as usize);
    };
}

macro_rules! get_nullable_f64 {
    ($self: expr, $target: ident, $vm: ident) => {
        match $self.data.borrow().$target {
            Some(data) => Ok($vm.ctx.new_float(data as f64).into()),
            None => Ok($vm.ctx.none().into())
        }
    };
}

/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
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
    pub fn get_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        get_string!(self, id, vm)
    }

    #[pymethod]
    pub fn set_id(&self, id: String) -> PyResult<()> {
        set_string!(self, id, id);
        Ok(())
    }

    #[pymethod]
    pub fn get_request_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        get_nullable_f64!(self, request_id, vm)
    }

    #[pymethod]
    pub fn set_request_id(&self, request_id: Option<PyIntRef>, _: &VirtualMachine) -> PyResult<()> {
        set_nullable_usize!(self, request_id, request_id);
        Ok(())
    }

    #[pymethod]
    pub fn get_device_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        get_string!(self, id, vm)
    }

    #[pymethod]
    pub fn set_device_id(&self, device_id: String) -> PyResult<()> {
        set_string!(self, id, device_id);
        Ok(())
    }
}

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* **************************************************************************************************************** */
pub trait ModelWrapper {
    type Entity;
    fn wrap(entity: Rc<RefCell<Self::Entity>>) -> Self;
}

/* **************************************************************************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* ************************************************* MACROS CALL ************************************************** */
/* **************************************************************************************************************** */
wrapper!(DeviceIdAuthRequest, DeviceIdAuthRequestWrapper);

/* **************************************************************************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */