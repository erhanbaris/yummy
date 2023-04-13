#[rustpython::vm::pymodule]
pub mod _room {
    use yummy_model::UserId;
    /* **************************************************************************************************************** */
    /* **************************************************** MODS ****************************************************** */
    /* *************************************************** IMPORTS **************************************************** */
    /* **************************************************************************************************************** */
    use yummy_model::{meta::RoomMetaAccess, RoomId};
    use rustpython_vm::{VirtualMachine, PyResult, PyObjectRef, function::OptionalArg};
    use rustpython_vm::class::PyClassImpl;

    use crate::plugin::python::{model::YummyPluginContextWrapper, util::MetaTypeUtil, modules::{model::model::RoomMetaAccessWrapper, base::_base::PyYummyValidationError}};

    /* **************************************************************************************************************** */
    /* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
    /* **************************************************** MACROS **************************************************** */
    /* *************************************************** STRUCTS **************************************************** */
    /* **************************************************** ENUMS ***************************************************** */
    /* ************************************************** FUNCTIONS *************************************************** */
    /* **************************************************************************************************************** */
    
    #[pyfunction]
    pub fn get_room_meta(room_id: Option<String>, key: Option<String>, vm: &VirtualMachine) -> PyResult<PyObjectRef> {

        /* Validate arguments */
        let (room_id, key) = match (room_id, key) {

            /* All arguments are valid */
            (Some(room_id), Some(key)) => (room_id, key),

            /* Return None if the arguments are not valid */
            _ => return Ok(vm.ctx.none())
        };

        /* Get plugin context from global variables */
        let context = vm.current_globals().get_item("__CONTEXT__", vm)?;
        let context = match context.payload::<YummyPluginContextWrapper>() {
            Some(context) => context,
            None => {
                log::error!("__CONTEXT__ information is null");
                return Ok(vm.ctx.none()); 
            }
        };

        match context.data.room_logic.get_room_meta(RoomId::from(room_id), key) {

            /* Room's meta found */
            Ok(Some(room_meta)) => MetaTypeUtil::as_python_value(&room_meta, vm),

            /* No meta for room */
            Ok(None) => Ok(vm.ctx.none()),

            /* Something went wrong, but do not throw exception. Only return None and log error message */
            Err(error) => {
                log::error!("Context is failed to retrieve 'get_room_meta'. Error: {}", error.to_string());
                Ok(vm.ctx.none())
            }
        }
    }
    
    #[pyfunction]
    pub fn get_room_metas(room_id: Option<String>, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        
        /* Validate arguments */
        let room_id = match room_id {

            /* All arguments are valid */
            Some(room_id) => room_id,

            /* Return None if the arguments are not valid */
            _ => return Ok(vm.ctx.none())
        };

        /* Get plugin context from global variables */
        let context = vm.current_globals().get_item("__CONTEXT__", vm)?;
        let context = match context.payload::<YummyPluginContextWrapper>() {
            Some(context) => context,
            None => {
                log::error!("__CONTEXT__ information is null");
                return Ok(vm.ctx.none()); 
            }
        };

        match context.data.room_logic.get_room_metas(RoomId::from(room_id)) {

            /* Room's metas found */
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
                log::error!("Context is failed to retrieve 'get_room_meta'. Error: {}", error.to_string());
                Ok(vm.ctx.none())
            }
        }
    }
    
    #[pyfunction]
    pub fn set_room_meta(room_id: Option<String>, key: Option<String>, value: PyObjectRef, access_level: OptionalArg<RoomMetaAccessWrapper>, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        
        /* Validate arguments */
        let (room_id, key) = match (room_id, key) {

            /* All arguments are valid */
            (Some(room_id), Some(key)) => (room_id, key),

            /* Return None if the arguments are not valid */
            _ => return Ok(vm.ctx.new_bool(false).into())
        };

        /* Configure meta access level */
        let access_level = match access_level {
            OptionalArg::Present(access_level) => access_level.data,
            OptionalArg::Missing => RoomMetaAccess::System
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
        let value = MetaTypeUtil::parse_room_meta(vm, &value, access_level)?;

        match context.data.room_logic.set_room_meta(RoomId::from(room_id), key, value.data) {

            /* Room's meta update/inserted, return True */
            Ok(_) => Ok(vm.ctx.new_bool(true).into()),

            /* Something went wrong, but do not throw exception. Only return False and log error message */
            Err(error) => {
                log::error!("Context is failed to retrieve 'set_room_meta'. Error: {}", error.to_string());
                Ok(vm.ctx.new_bool(false).into())
            }
        }
    }

    #[pyfunction]
    pub fn remove_room_metas(room_id: Option<String>, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        
        /* Validate arguments */
        let room_id = match room_id {

            /* All arguments are valid */
            Some(room_id) => room_id,

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

        match context.data.room_logic.remove_all_metas(RoomId::from(room_id)) {

            /* Room's metas removed */
            Ok(_) => Ok(vm.ctx.new_bool(true).into()),

            /* Something went wrong, but do not throw exception. Only return None and log error message */
            Err(error) => {
                log::error!("Context is failed to retrieve 'remove_room_metas'. Error: {}", error.to_string());
                Ok(vm.ctx.new_bool(false).into())
            }
        }
    }

    #[pyfunction]
    pub fn remove_room_meta(room_id: Option<String>, key: Option<String>, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        /* Validate arguments */
        let (room_id, key) = match (room_id, key) {

            /* All arguments are valid */
            (Some(room_id), Some(key)) => (room_id, key),

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

        match context.data.room_logic.remove_room_meta(RoomId::from(room_id), key) {

            /* Room's metas removed */
            Ok(_) => Ok(vm.ctx.new_bool(true).into()),

            /* Something went wrong, but do not throw exception. Only return None and log error message */
            Err(error) => {
                log::error!("Context is failed to retrieve 'remove_room_meta'. Error: {}", error.to_string());
                Ok(vm.ctx.new_bool(false).into())
            }
        }
    }

    #[pyfunction]
    pub fn message_to_room(room_id: Option<String>, message: PyObjectRef, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        /* Validate arguments */
        let room_id = match room_id {

            /* All arguments are valid */
            Some(room_id) => room_id,

            /* Return None if the arguments are not valid */
            _ => return Ok(vm.ctx.new_bool(false).into())
        };

        let room_id = RoomId::from(room_id);

        /* Get plugin context from global variables */
        let context = vm.current_globals().get_item("__CONTEXT__", vm)?;
        let context = match context.payload::<YummyPluginContextWrapper>() {
            Some(context) => context,
            None => {
                log::error!("__CONTEXT__ information is null");
                return Ok(vm.ctx.new_bool(false).into()); 
            }
        };

        let obj_serializer = rustpython_vm::py_serde::PyObjectSerializer::new(vm, &message);
        let message = match serde_json::value::to_value(obj_serializer) {
            Ok(message) => message,
            Err(error) => return Err(vm.new_exception_msg(PyYummyValidationError::make_class(&vm.ctx), error.to_string()))
        };

        match context.data.room_logic.message_to_room(&room_id, None, &message) {

            /* Message sent to room users */
            Ok(_) => Ok(vm.ctx.new_bool(true).into()),

            /* Something went wrong, but do not throw exception. Only return None and log error message */
            Err(error) => {
                log::error!("Context is failed to retrieve 'message_to_room'. Error: {}", error.to_string());
                Ok(vm.ctx.new_bool(false).into())
            }
        }
    }

    #[pyfunction]
    pub fn message_to_room_user(room_id: Option<String>, user_id: Option<String>, message: PyObjectRef, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        /* Validate arguments */
        let (room_id, user_id) = match (room_id, user_id) {

            /* All arguments are valid */
            (Some(room_id), Some(user_id)) => (room_id, user_id),

            /* Return None if the arguments are not valid */
            _ => return Ok(vm.ctx.new_bool(false).into())
        };

        let room_id = RoomId::from(room_id);
        let user_id = UserId::from(user_id);

        /* Get plugin context from global variables */
        let context = vm.current_globals().get_item("__CONTEXT__", vm)?;
        let context = match context.payload::<YummyPluginContextWrapper>() {
            Some(context) => context,
            None => {
                log::error!("__CONTEXT__ information is null");
                return Ok(vm.ctx.new_bool(false).into()); 
            }
        };

        let obj_serializer = rustpython_vm::py_serde::PyObjectSerializer::new(vm, &message);
        let message = match serde_json::value::to_value(obj_serializer) {
            Ok(message) => message,
            Err(error) => return Err(vm.new_exception_msg(PyYummyValidationError::make_class(&vm.ctx), error.to_string()))
        };

        match context.data.room_logic.message_to_room_user(&room_id, &user_id, None, &message) {

            /* Message sent to room users */
            Ok(_) => Ok(vm.ctx.new_bool(true).into()),

            /* Something went wrong, but do not throw exception. Only return None and log error message */
            Err(error) => {
                log::error!("Context is failed to retrieve 'message_to_room_room'. Error: {}", error.to_string());
                Ok(vm.ctx.new_bool(false).into())
            }
        }
    }

    /* **************************************************************************************************************** */
    /* *************************************************** TRAITS ***************************************************** */
    /* ************************************************* IMPLEMENTS *************************************************** */
    /* ********************************************** TRAIT IMPLEMENTS ************************************************ */
    /* ************************************************* MACROS CALL ************************************************** */
    /* ************************************************** UNIT TESTS ************************************************** */
    /* **************************************************************************************************************** */
}
