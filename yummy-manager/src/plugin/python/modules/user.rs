#[rustpython::vm::pymodule]
pub mod _user {
    /* **************************************************************************************************************** */
    /* **************************************************** MODS ****************************************************** */
    /* *************************************************** IMPORTS **************************************************** */
    /* **************************************************************************************************************** */
    use yummy_model::{meta::UserMetaAccess, UserId};
    use rustpython_vm::{VirtualMachine, PyResult, PyObjectRef, function::OptionalArg};

    use crate::plugin::python::{model::YummyPluginContextWrapper, util::MetaTypeUtil, modules::model::_model::UserMetaAccessWrapper};

    /* **************************************************************************************************************** */
    /* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
    /* **************************************************** MACROS **************************************************** */
    /* *************************************************** STRUCTS **************************************************** */
    /* **************************************************** ENUMS ***************************************************** */
    /* ************************************************** FUNCTIONS *************************************************** */
    /* **************************************************************************************************************** */
    
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
    /* ********************************************** TRAIT IMPLEMENTS ************************************************ */
    /* ************************************************* MACROS CALL ************************************************** */
    /* ************************************************** UNIT TESTS ************************************************** */
    /* **************************************************************************************************************** */
}
