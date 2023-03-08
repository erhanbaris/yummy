/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
use rustpython_vm::{convert::ToPyObject, object::PyWeak, class::PyClassImpl};
use rustpython_vm::{builtins::PyList, PyObjectRef};
use rustpython_vm::{
    pyclass, pymodule, PyObject, PyPayload, PyResult, TryFromBorrowedObject, VirtualMachine,
};

use crate::auth::model::DeviceIdAuthRequest;

/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* *************************************************** STRUCTS **************************************************** */
#[pymodule]
pub mod rust_py_module {
    use rustpython_vm::{convert::ToPyObject, object::PyWeak, class::PyClassImpl};
    use rustpython_vm::{builtins::PyList, PyObjectRef};
    use rustpython_vm::{
        pyclass, pymodule, PyObject, PyPayload, PyResult, TryFromBorrowedObject, VirtualMachine,
    };

    use crate::auth::model::DeviceIdAuthRequest;
    #[pyattr]
    #[pyclass(module = false, name = "DeviceIdAuth")]
    #[derive(Debug, PyPayload)]
    pub struct DeviceIdAuthRequestWrapper(pub DeviceIdAuthRequest);

    #[pyclass]
    impl DeviceIdAuthRequestWrapper {
        #[pymethod]
        pub fn device_id(&self) -> String {
            self.0.id.clone()
        }
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