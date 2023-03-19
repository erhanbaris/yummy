#[rustpython::vm::pymodule]
pub mod yummy {
    /* **************************************************************************************************************** */
    /* **************************************************** MODS ****************************************************** */
    /* *************************************************** IMPORTS **************************************************** */
    /* **************************************************************************************************************** */
    use std::ops::Deref;
    use std::{rc::Rc, cell::RefCell};
    use std::fmt::Debug;

    use yummy_macros::yummy_model;
    use yummy_model::UserId;
    use yummy_model::meta::{UserMetaType, UserMetaAccess};
    use rustpython::vm::{pyclass, PyPayload};
    use rustpython::vm::function::OptionalArg;
    use rustpython::vm::{TryFromBorrowedObject, PyRef, PyObject};
    use rustpython::vm::builtins::{PyBaseException, PyInt};
    use rustpython::vm::{builtins::{PyBaseExceptionRef, PyIntRef}, VirtualMachine, PyResult, PyObjectRef};

    use crate::auth::model::{CustomIdAuthRequest, LogoutRequest, ConnUserDisconnect, RefreshTokenRequest};
    use crate::conn::model::UserConnected;
    use crate::plugin::python::util::MetaTypeUtil;
    use crate::{plugin::python::model::YummyPluginContextWrapper, auth::model::{DeviceIdAuthRequest, EmailAuthRequest}};
    use crate::plugin::python::ModelWrapper;

    use yummy_general::password::Password;

    /* **************************************************************************************************************** */
    /* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
    /* **************************************************** MACROS **************************************************** */
    /* **************************************************************************************************************** */

    macro_rules! wrapper_struct {
        ($model: ident, $wrapper: ident, $class_name: expr) => {
            #[pyclass(module = "yummy", name = $class_name)]
            #[derive(Debug, PyPayload)]
            pub struct $wrapper {
                pub data: Rc<RefCell< $model >>
            }

            impl ModelWrapper for $wrapper {
                type Entity = $model;
                fn wrap(entity: Rc<RefCell<Self::Entity>>) -> Self {
                    $wrapper::new(entity)
                }
            }

            unsafe impl Send for $wrapper {}
            unsafe impl Sync for $wrapper {}
        }
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

    macro_rules! get_user_id {
        ($self: expr, $vm: ident) => {
            match $self.data.borrow().auth.deref() {
                Some(auth) => Ok($vm.ctx.new_str(&auth.user.to_string()[..]).into()),
                None => Ok($vm.ctx.none().into())
            }
        };
    }

    macro_rules! get_session_id {
        ($self: expr, $vm: ident) => {
            match $self.data.borrow().auth.deref() {
                Some(auth) => Ok($vm.ctx.new_str(&auth.session.to_string()[..]).into()),
                None => Ok($vm.ctx.none().into())
            }
        };
    }

    /* **************************************************************************************************************** */
    /* *************************************************** STRUCTS **************************************************** */
    /* **************************************************************************************************************** */
    #[pyattr]
    #[pyclass(module = "yummy", name = "YummyValidationError")]
    //#[pyexception(PyYummyValidationError, PyBaseException)]
    #[derive(PyPayload, Debug)]
    pub struct PyYummyValidationError {}

    wrapper_struct!(DeviceIdAuthRequest, DeviceIdAuthRequestWrapper, "DeviceIdAuth");
    wrapper_struct!(EmailAuthRequest, EmailAuthRequestWrapper, "EmailAuth");
    wrapper_struct!(CustomIdAuthRequest, CustomIdAuthRequestWrapper, "CustomIdAuth");
    wrapper_struct!(UserConnected, UserConnectedWrapper, "UserConnected");
    wrapper_struct!(ConnUserDisconnect, ConnUserDisconnectWrapper, "ConnUserDisconnect");
    wrapper_struct!(LogoutRequest, LogoutRequestWrapper, "Logout");
    wrapper_struct!(RefreshTokenRequest, RefreshTokenRequestWrapper, "RefreshToken");

    #[pyattr]
    #[pyclass(module = false, name = "UserMetaType")]
    #[derive(Debug, PyPayload)]
    pub struct UserMetaTypeWrapper {
        pub data: UserMetaType
    }

    #[pyattr]
    #[pyclass(module = false, name = "UserMetaAccess")]
    #[derive(Debug, PyPayload)]
    pub struct UserMetaAccessWrapper {
        pub data: UserMetaAccess
    }
 
    /* **************************************************************************************************************** */
    /* **************************************************** ENUMS ***************************************************** */
    /* ************************************************** FUNCTIONS *************************************************** */
    /* **************************************************************************************************************** */

    #[pyfunction]
    pub fn fail(message: String, vm: &VirtualMachine) -> PyResult<PyBaseExceptionRef> {
        use rustpython::vm::class::PyClassImpl;
        Err(vm.new_exception_msg(PyYummyValidationError::make_class(&vm.ctx), message))
    }
    
    #[pyfunction]
    pub fn get_user_meta(user_id: Option<String>, key: Option<String>, vm: &VirtualMachine) -> PyResult<PyObjectRef> {

        /* Validate arguments */
        let (user_id, key) = match (user_id, key) {

            /* All arguments are valid */
            (Some(user_id), Some(key)) => (user_id, key),

            /* Return None if the arguments are not valid */
            _ => return Ok(vm.ctx.none().into())
        };

        /* Get plugin context from global variables */
        let context = vm.current_globals().get_item("__CONTEXT__", vm)?;
        let context = match context.payload::<YummyPluginContextWrapper>() {
            Some(context) => context,
            None => {
                log::error!("__CONTEXT__ information is null");
                return Ok(vm.ctx.none().into()); 
            }
        };

        match context.data.user_logic.get_user_meta(UserId::from(user_id), key) {

            /* User's meta found */
            Ok(Some(user_meta)) => MetaTypeUtil::as_python_value(&user_meta, vm),

            /* No meta for user */
            Ok(None) => Ok(vm.ctx.none().into()),

            /* Something went wrong, but do not throw exception. Only return None and log error message */
            Err(error) => {
                log::error!("Context is failed to retrieve 'get_user_meta'. Error: {}", error.to_string());
                Ok(vm.ctx.none().into())
            }
        }
    }
    
    #[pyfunction]
    pub fn get_user_metas(user_id: Option<String>, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        
        /* Validate arguments */
        let user_id = match user_id {

            /* All arguments are valid */
            Some(user_id) => user_id,

            /* Return None if the arguments are not valid */
            _ => return Ok(vm.ctx.none().into())
        };

        /* Get plugin context from global variables */
        let context = vm.current_globals().get_item("__CONTEXT__", vm)?;
        let context = match context.payload::<YummyPluginContextWrapper>() {
            Some(context) => context,
            None => {
                log::error!("__CONTEXT__ information is null");
                return Ok(vm.ctx.none().into()); 
            }
        };

        match context.data.user_logic.get_user_metas(UserId::from(user_id)) {

            /* User's metas found */
            Ok(metas) => {

                let python_dict = vm.ctx.new_dict();
                
                for meta in metas.into_iter() {
                    /* Convert meta type to python object and add into the vector */
                    python_dict.set_item(&meta.name[..], MetaTypeUtil::as_python_value(&meta.meta, vm)?, vm)?;
                }

                Ok(python_dict.into())
            },

            /* Something went wrong, but do not throw exception. Only return None and log error message */
            Err(error) => {
                log::error!("Context is failed to retrieve 'get_user_meta'. Error: {}", error.to_string());
                Ok(vm.ctx.none().into())
            }
        }
    }
    
    #[pyfunction]
    pub fn set_user_meta(user_id: Option<String>, key: Option<String>, value: PyObjectRef, access_level: OptionalArg<UserMetaAccessWrapper>, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        
        /* Validate arguments */
        let (user_id, key) = match (user_id, key) {

            /* All arguments are valid */
            (Some(user_id), Some(key)) => (user_id, key),

            /* Return None if the arguments are not valid */
            _ => return Ok(vm.ctx.new_bool(false).into())
        };

        /* Configure meta access level */
        let access_level = match access_level {
            OptionalArg::Present(access_level) => access_level.data,
            OptionalArg::Missing => UserMetaAccess::System
        };

        /* Get plugin context from global variables */
        let context = vm.current_globals().get_item("__CONTEXT__", vm)?;
        let context = match context.payload::<YummyPluginContextWrapper>() {
            Some(context) => context,
            None => {
                log::error!("__CONTEXT__ information is null");
                return Ok(vm.ctx.new_bool(false).into())
            }
        };

        /* Build meta information */
        let value = MetaTypeUtil::parse(vm, &value, access_level)?;

        match context.data.user_logic.set_user_meta(UserId::from(user_id), key, value.data) {

            /* User's meta update/inserted, return True */
            Ok(_) => Ok(vm.ctx.new_bool(true).into()),

            /* Something went wrong, but do not throw exception. Only return False and log error message */
            Err(error) => {
                log::error!("Context is failed to retrieve 'set_user_meta'. Error: {}", error.to_string());
                Ok(vm.ctx.new_bool(false).into())
            }
        }
    }

    #[pyfunction]
    pub fn remove_user_metas(user_id: Option<String>, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        
        /* Validate arguments */
        let user_id = match user_id {

            /* All arguments are valid */
            Some(user_id) => user_id,

            /* Return None if the arguments are not valid */
            _ => return Ok(vm.ctx.new_bool(false).into())
        };

        /* Get plugin context from global variables */
        let context = vm.current_globals().get_item("__CONTEXT__", vm)?;
        let context = match context.payload::<YummyPluginContextWrapper>() {
            Some(context) => context,
            None => {
                log::error!("__CONTEXT__ information is null");
                return Ok(vm.ctx.new_bool(false).into()); 
            }
        };

        match context.data.user_logic.remove_all_metas(UserId::from(user_id)) {

            /* User's metas removed */
            Ok(_) => Ok(vm.ctx.new_bool(true).into()),

            /* Something went wrong, but do not throw exception. Only return None and log error message */
            Err(error) => {
                log::error!("Context is failed to retrieve 'remove_user_metas'. Error: {}", error.to_string());
                Ok(vm.ctx.new_bool(false).into())
            }
        }
    }

    #[pyfunction]
    pub fn remove_user_meta(user_id: Option<String>, key: Option<String>, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        
        /* Validate arguments */
        let (user_id, key) = match (user_id, key) {

            /* All arguments are valid */
            (Some(user_id), Some(key)) => (user_id, key),

            /* Return None if the arguments are not valid */
            _ => return Ok(vm.ctx.new_bool(false).into())
        };

        /* Get plugin context from global variables */
        let context = vm.current_globals().get_item("__CONTEXT__", vm)?;
        let context = match context.payload::<YummyPluginContextWrapper>() {
            Some(context) => context,
            None => {
                log::error!("__CONTEXT__ information is null");
                return Ok(vm.ctx.new_bool(false).into()); 
            }
        };

        match context.data.user_logic.remove_user_meta(UserId::from(user_id), key) {

            /* User's metas removed */
            Ok(_) => Ok(vm.ctx.new_bool(true).into()),

            /* Something went wrong, but do not throw exception. Only return None and log error message */
            Err(error) => {
                log::error!("Context is failed to retrieve 'remove_user_meta'. Error: {}", error.to_string());
                Ok(vm.ctx.new_bool(false).into())
            }
        }
    }

    /* **************************************************************************************************************** */
    /* *************************************************** TRAITS ***************************************************** */
    /* ************************************************* IMPLEMENTS *************************************************** */
    /* **************************************************************************************************************** */
    #[pyclass]
    impl PyYummyValidationError { }

    #[yummy_model(class_name="UserConnected", no_request_id=true, no_auth=true)]
    #[pyclass(flags(BASETYPE))]
    impl UserConnectedWrapper {
        #[pymethod]
        pub fn get_user_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_str(&self.data.borrow_mut().user_id.to_string()[..]).into())
        }
    }

    #[yummy_model(class_name="RefreshTokenRequest")]
    #[pyclass(flags(BASETYPE))]
    impl RefreshTokenRequestWrapper {
        #[pymethod]
        pub fn get_token(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_str(&self.data.borrow_mut().token.to_string()[..]).into())
        }
    }

    #[yummy_model(class_name="ConnUserDisconnect", no_request_id=true, no_auth=true)]
    #[pyclass(flags(BASETYPE))]
    impl ConnUserDisconnectWrapper {}

    #[yummy_model(class_name="LogoutRequest")]
    #[pyclass(flags(BASETYPE))]
    impl LogoutRequestWrapper {}

    #[yummy_model(class_name="DeviceIdAuthRequest")]
    #[pyclass(flags(BASETYPE))]
    impl DeviceIdAuthRequestWrapper {
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

    #[yummy_model(class_name="EmailAuthRequest")]
    #[pyclass(flags(BASETYPE))]
    impl EmailAuthRequestWrapper {

        #[pymethod]
        pub fn get_email(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_string!(self, email, vm)
        }

        #[pymethod]
        pub fn set_email(&self, email: String) -> PyResult<()> {
            set_string!(self, email, email);
            Ok(())
        }

        #[pymethod]
        pub fn get_password(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_str(&self.data.borrow_mut().password.get()[..]).into())
        }

        #[pymethod]
        pub fn set_password(&self, password: String) -> PyResult<()> {
            self.data.borrow_mut().password = Password::from(password);
            Ok(())
        }
    }

    #[yummy_model(class_name="CustomIdAuthRequest")]
    #[pyclass(flags(BASETYPE))]
    impl CustomIdAuthRequestWrapper {
        #[pymethod]
        pub fn get_custom_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_string!(self, id, vm)
        }

        #[pymethod]
        pub fn set_custom_id(&self, device_id: String) -> PyResult<()> {
            set_string!(self, id, device_id);
            Ok(())
        }
    }

    /* ################################################# DeviceIdAuth ################################################# */

    /* ################################################## EmailAuth ################################################### */

    /* ################################################ CustomIdAuth ################################################## */

    /* ################################################# Logout ################################################### */

    /* ########################################### UserMetaTypeWrapper ################################################# */
    #[pyclass(flags(BASETYPE))]
    impl UserMetaTypeWrapper {
        pub fn new(data: UserMetaType) -> Self {
            Self { data }
        }
    }

    #[pyclass(flags(BASETYPE))]
    impl UserMetaAccessWrapper {
        pub fn new(data: UserMetaAccess) -> Self {
            Self { data }
        }
    }

    /* **************************************************************************************************************** */
    /* ********************************************** TRAIT IMPLEMENTS ************************************************ */
    /* **************************************************************************************************************** */

    impl TryFromBorrowedObject for UserMetaAccessWrapper {
        fn try_from_borrowed_object(vm: &VirtualMachine, obj: &PyObject) -> Result<Self, PyRef<PyBaseException>> {
            if obj.class().fast_issubclass(vm.ctx.types.int_type) {
                return Ok(UserMetaAccessWrapper::new(UserMetaAccess::from(obj.payload::<PyInt>().unwrap().as_u32_mask() as i32)));
            }

            Ok(UserMetaAccessWrapper { data: UserMetaAccess::System })
        }
    }

    /* **************************************************************************************************************** */
    /* ************************************************* MACROS CALL ************************************************** */
    /* ************************************************** UNIT TESTS ************************************************** */
    /* **************************************************************************************************************** */
}