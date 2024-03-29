/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* **************************************************************************************************************** */
#[cfg(test)]
mod test;
mod model;
mod modules;
mod util;

/* **************************************************************************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::{cell::RefCell, path::Path};
use std::rc::Rc;

use std::sync::Arc;
use std::fs;

use glob::{MatchOptions, glob_with};
use yummy_model::config::YummyConfig;
use rustpython::{vm as vm, InterpreterConfig};
use strum::IntoEnumIterator;
use vm::convert::ToPyObject;
use vm::function::IntoFuncArgs;
use vm::scope::Scope;
use vm::{VirtualMachine, PyObjectRef, AsObject};
use strum_macros::EnumIter;

use std::collections::HashMap;
use std::ops::Deref;

use crate::plugin::python::model::YummyPluginContextWrapper;
use crate::plugin::python::modules::model::model::{DeviceIdAuthRequestWrapper, EmailAuthRequestWrapper, CustomIdAuthRequestWrapper, LogoutRequestWrapper, UserConnectedWrapper, ConnUserDisconnectWrapper, RefreshTokenRequestWrapper, RestoreTokenRequestWrapper, GetUserInformationWrapper, UpdateUserWrapper, CreateRoomRequestWrapper};
use crate::room::model::Play;
use crate::{
    auth::model::{ConnUserDisconnect, CustomIdAuthRequest, DeviceIdAuthRequest, EmailAuthRequest, LogoutRequest, RefreshTokenRequest, RestoreTokenRequest},
    conn::model::UserConnected,
    room::model::{
        CreateRoomRequest, DisconnectFromRoomRequest, GetRoomRequest, JoinToRoomRequest, KickUserFromRoom, MessageToRoomRequest, ProcessWaitingUser, RoomListRequest, UpdateRoom, WaitingRoomJoins,
    },
    user::model::{GetUserInformation, UpdateUser},
};
use self::model::ModelWrapper;
use self::modules::configure_modules;
use self::modules::model::model::{UpdateRoomWrapper, JoinToRoomRequestWrapper, ProcessWaitingUserWrapper, KickUserFromRoomWrapper, DisconnectFromRoomRequestWrapper, MessageToRoomRequestWrapper, RoomListRequestWrapper, WaitingRoomJoinsWrapper, GetRoomRequestWrapper, PlayWrapper};

use super::{YummyPlugin, YummyPluginInstaller, YummyPluginError, PluginExecuter};

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* **************************************************************************************************************** */
macro_rules! create_func {
    ($pre: ident, $post: ident, $function: path, $model: path, $wrapper: tt) => {
        fn $pre<'a>(&self, model: Rc<RefCell<$model>>) -> Result<(), YummyPluginError> {
            self.execute_pre_functions::<_, $wrapper>(model, $function)
        }

        fn $post<'a>(&self, model: Rc<RefCell<$model>>, success: bool) -> Result<(), YummyPluginError> {
            self.execute_post_functions::<_, $wrapper>(model, success, $function)
        }
    };
}

/* **************************************************************************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
#[derive(Default)]
pub struct PythonPluginInstaller;

pub struct PythonPlugin {
    pub interpreter: vm::Interpreter,
    pub scopes: Vec<Scope>,
    pub pre_function_refs: HashMap<FunctionType, Vec<PyObjectRef>>,
    pub post_function_refs: HashMap<FunctionType, Vec<PyObjectRef>>
}

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* **************************************************************************************************************** */
#[derive(EnumIter, Eq, Hash, PartialEq, Copy, Clone)]
pub enum FunctionType {
    EmailAuth,
    DeviceidAuth,
    CustomidAuth,
    Logout,
    RefreshToken,
    RestoreToken,
    UserConnected,
    UserDisconnected,
    GetUserInformation,
    UpdateUser,
    CreateRoom,
    UpdateRoom,
    JoinToRoom,
    ProcessWaitingUser,
    KickUserFromRoom,
    DisconnectFromRoomRequest,
    MessageToRoomRequest,
    RoomListRequest,
    WaitingRoomJoins,
    GetRoomRequest,
    Play
}

/* **************************************************************************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* **************************************************************************************************************** */
impl PythonPlugin {
    pub fn execute_pre_functions<T, W: ToPyObject + rustpython::vm::PyPayload + ModelWrapper<Entity = T> + 'static>(&self, model: Rc<RefCell<T>>, function: FunctionType) -> Result<(), YummyPluginError> {
        self.interpreter.enter(|vm| {
            let model = W::wrap(model).to_pyobject(vm);
            self.inner_execute(vm, &self.pre_function_refs, (model, ), function)
        })
    }

    pub fn execute_post_functions<T, W: ToPyObject + rustpython::vm::PyPayload + ModelWrapper<Entity = T> + 'static>(&self, model: Rc<RefCell<T>>, success: bool, function: FunctionType) -> Result<(), YummyPluginError> {
        self.interpreter.enter(|vm| {
            let model = W::wrap(model).to_pyobject(vm);
            self.inner_execute(vm, &self.post_function_refs, (model, success), function)
        })
    }

    fn inner_execute(&self, vm: &VirtualMachine, functions: &HashMap<FunctionType, Vec<PyObjectRef>>, args: impl IntoFuncArgs + Clone, function: FunctionType) -> Result<(), YummyPluginError> {

        // Get all parsed functions references to invoke
        if let Some(functions) = functions.get(&function) {

            // Iterate over the functions
            for function in functions.iter() {

                // Call the function with our model and check everyting went well
                if let Err(error) = function.call(args.clone(), vm) {

                    /*
                    We should return an error message instead of the exception object.
                    Our error message requires post-processing
                    */
                    
                    if error.class().name().deref() == "YummyValidationError" {
                        let message = match error.get_arg(0) {
                            Some(arg) => match arg.str(vm) {
                                Ok(arg) => arg.as_str().to_string(),
                                Err(_) => "Python validation error".to_string()
                            },
                            None => "Python validation error".to_string()
                        };
                        return Err(YummyPluginError::Validation(message));
                    }
                    
                    let mut error_message = String::new();
                    log::error!("Python scripting error: {}", error_message);
                    vm.write_exception(&mut error_message, &error).unwrap();
                    return Err(YummyPluginError::Internal(error_message));
                }
            }
        }

        Ok(())
    }
}

impl PythonPluginInstaller {
    pub fn build_plugin(executer: &PluginExecuter, config: Arc<YummyConfig>) -> PythonPlugin {
        let mut scopes = Vec::new();
        let mut pre_function_refs: HashMap<FunctionType, Vec<PyObjectRef>> = HashMap::new();
        let mut post_function_refs: HashMap<FunctionType, Vec<PyObjectRef>> = HashMap::new();

        let interpreter = InterpreterConfig::new()
            .init_stdlib()
            .init_hook(Box::new(|vm| {
                vm.add_native_module("yummy".to_owned(), Box::new(configure_modules));
            }))
            .interpreter();

        interpreter
            .enter(|vm| -> vm::PyResult<()> {
                
                crate::plugin::python::modules::model::make_module(vm);

                let mut build = || {
                    let path = Path::new(&config.python_files_path).join("*.py").to_string_lossy().to_string();
                    log::info!("Searhing python files at {}", path);
            
                    let options = MatchOptions {
                        case_sensitive: false,
                        require_literal_separator: false,
                        require_literal_leading_dot: false,
                    };
            
                    let paths = glob_with(&path, options).unwrap();
                    for path in paths {
                        let path = path.unwrap().to_string_lossy().to_string();
                        let content = fs::read_to_string(&path).unwrap();

                        // Read python script and parse the codes
                        vm.insert_sys_path(vm.new_pyobj(&config.python_files_path))?;
                        let code_obj = vm.compile(&content, vm::compiler::Mode::Exec, path.clone(),)
                            .map_err(|err| vm.new_syntax_error(&err))?;
                        
                        // Create new scope for python script. It will contains locals and globals
                        let scope = vm.new_scope_with_builtins();
                        
                        // Compile and run parsed python codes. We want to use scope later on
                        if let Err(error) = vm.run_code_obj(code_obj, scope.clone()) {
                            let mut error_message = String::new();
                            vm.write_exception(&mut error_message, &error).unwrap();
                            log::error!("'{}' failed to compile. Error message: {}", path, error_message);
                            return Err(error);
                        }

                        scope.globals.set_item("__CONTEXT__", YummyPluginContextWrapper::new(executer.context.clone()).to_pyobject(vm), vm).unwrap();

                        /*
                        Build python method informations to call it later.
                        That approach will increase function invoke performance
                        */
                        for function_type in FunctionType::iter() {
                            // Get pre execution functions from python script
                            if let Ok(function_ref) = scope.globals.get_item(function_type.get_pre_function_name(), vm) {
                                let functions = pre_function_refs.entry(function_type).or_insert(Vec::new());
                                functions.push(function_ref);
                            }

                            // Get post execution functions from python script
                            if let Ok(function_ref) = scope.globals.get_item(function_type.get_post_function_name(), vm) {
                                let functions = post_function_refs.entry(function_type).or_insert(Vec::new());
                                functions.push(function_ref);
                            }
                        }

                        scopes.push(scope);
                    }

                    Ok(())
                };

                match build() {
                    Ok(_) => Ok(()),
                    Err(error) => {
                        let mut error_message = String::new();
                        vm.write_exception(&mut error_message, &error).unwrap();
                        log::error!("Error message: {}", error_message);
                        Err(error)
                    }
                }
            }).unwrap();

        PythonPlugin {
            interpreter,
            scopes,
            pre_function_refs,
            post_function_refs
        }
    }
}

impl FunctionType {
    pub fn get_pre_function_name(&self) -> &'static str {
        match self {
            FunctionType::EmailAuth => "pre_email_auth",
            FunctionType::DeviceidAuth => "pre_deviceid_auth",
            FunctionType::CustomidAuth => "pre_customid_auth",
            FunctionType::Logout => "pre_logout",
            FunctionType::RefreshToken => "pre_refresh_token",
            FunctionType::RestoreToken => "pre_restore_token",
            FunctionType::UserConnected => "pre_user_connected",
            FunctionType::UserDisconnected => "pre_user_disconnected",
            FunctionType::GetUserInformation => "pre_get_user_information",
            FunctionType::UpdateUser => "pre_update_user",
            FunctionType::CreateRoom => "pre_create_room",
            FunctionType::UpdateRoom => "pre_update_room",
            FunctionType::JoinToRoom => "pre_join_to_room",
            FunctionType::ProcessWaitingUser => "pre_process_waiting_user",
            FunctionType::KickUserFromRoom => "pre_kick_user_from_room",
            FunctionType::DisconnectFromRoomRequest => "pre_disconnect_from_room",
            FunctionType::MessageToRoomRequest => "pre_message_to_room",
            FunctionType::RoomListRequest => "pre_room_list_request",
            FunctionType::WaitingRoomJoins => "pre_waiting_room_joins",
            FunctionType::GetRoomRequest => "pre_get_room_request",
            FunctionType::Play => "pre_play",
        }
    }

    pub fn get_post_function_name(&self) -> &'static str {
        match self {
            FunctionType::EmailAuth => "post_email_auth",
            FunctionType::DeviceidAuth => "post_deviceid_auth",
            FunctionType::CustomidAuth => "post_customid_auth",
            FunctionType::Logout => "post_logout",
            FunctionType::RefreshToken => "post_refresh_token",
            FunctionType::RestoreToken => "post_restore_token",
            FunctionType::UserConnected => "post_user_connected",
            FunctionType::UserDisconnected => "post_user_disconnected",
            FunctionType::GetUserInformation => "post_get_user_information",
            FunctionType::UpdateUser => "post_update_user",
            FunctionType::CreateRoom => "post_create_room",
            FunctionType::UpdateRoom => "post_update_room",
            FunctionType::JoinToRoom => "post_join_to_room",
            FunctionType::ProcessWaitingUser => "post_process_waiting_user",
            FunctionType::KickUserFromRoom => "post_kick_user_from_room",
            FunctionType::DisconnectFromRoomRequest => "post_disconnect_from_room",
            FunctionType::MessageToRoomRequest => "post_message_to_room",
            FunctionType::RoomListRequest => "post_room_list_request",
            FunctionType::WaitingRoomJoins => "post_waiting_room_joins",
            FunctionType::GetRoomRequest => "post_get_room_request",
            FunctionType::Play => "post_play",
        }
    }
}

/* **************************************************************************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* **************************************************************************************************************** */
impl YummyPlugin for PythonPlugin {
    // Auth manager
    create_func!(pre_email_auth, post_email_auth, FunctionType::EmailAuth, EmailAuthRequest, EmailAuthRequestWrapper);
    create_func!(pre_deviceid_auth, post_deviceid_auth, FunctionType::DeviceidAuth, DeviceIdAuthRequest, DeviceIdAuthRequestWrapper);
    create_func!(pre_customid_auth, post_customid_auth, FunctionType::CustomidAuth, CustomIdAuthRequest, CustomIdAuthRequestWrapper);
    create_func!(pre_logout, post_logout, FunctionType::Logout, LogoutRequest, LogoutRequestWrapper);
    create_func!(pre_refresh_token, post_refresh_token, FunctionType::RefreshToken, RefreshTokenRequest, RefreshTokenRequestWrapper);
    create_func!(pre_restore_token, post_restore_token, FunctionType::RestoreToken, RestoreTokenRequest, RestoreTokenRequestWrapper);

    // Connection manager
    create_func!(pre_user_connected, post_user_connected, FunctionType::UserConnected, UserConnected, UserConnectedWrapper);
    create_func!(pre_user_disconnected, post_user_disconnected, FunctionType::UserDisconnected, ConnUserDisconnect, ConnUserDisconnectWrapper);

    // User manager
    create_func!(pre_get_user_information, post_get_user_information, FunctionType::GetUserInformation, GetUserInformation, GetUserInformationWrapper);
    create_func!(pre_update_user, post_update_user, FunctionType::UpdateUser, UpdateUser, UpdateUserWrapper);

    // Room Manager
    create_func!(pre_create_room, post_create_room, FunctionType::CreateRoom, CreateRoomRequest, CreateRoomRequestWrapper);
    create_func!(pre_update_room, post_update_room, FunctionType::UpdateRoom, UpdateRoom, UpdateRoomWrapper);
    create_func!(pre_join_to_room, post_join_to_room, FunctionType::JoinToRoom, JoinToRoomRequest, JoinToRoomRequestWrapper);
    create_func!(pre_process_waiting_user, post_process_waiting_user, FunctionType::ProcessWaitingUser, ProcessWaitingUser, ProcessWaitingUserWrapper);
    create_func!(pre_kick_user_from_room, post_kick_user_from_room, FunctionType::KickUserFromRoom, KickUserFromRoom, KickUserFromRoomWrapper);
    create_func!(pre_disconnect_from_room, post_disconnect_from_room, FunctionType::DisconnectFromRoomRequest, DisconnectFromRoomRequest, DisconnectFromRoomRequestWrapper);
    create_func!(pre_message_to_room, post_message_to_room, FunctionType::MessageToRoomRequest, MessageToRoomRequest, MessageToRoomRequestWrapper);
    create_func!(pre_room_list_request, post_room_list_request, FunctionType::RoomListRequest, RoomListRequest, RoomListRequestWrapper);
    create_func!(pre_waiting_room_joins, post_waiting_room_joins, FunctionType::WaitingRoomJoins, WaitingRoomJoins, WaitingRoomJoinsWrapper);
    create_func!(pre_get_room_request, post_get_room_request, FunctionType::GetRoomRequest, GetRoomRequest, GetRoomRequestWrapper);
    create_func!(pre_play, post_play, FunctionType::Play, Play, PlayWrapper);
}

impl YummyPluginInstaller for PythonPluginInstaller {
    fn install(&self, executer: &mut PluginExecuter, config: Arc<YummyConfig>) {
        log::info!("Python plugin installing");

        let plugin = PythonPluginInstaller::build_plugin(executer, config);
        executer.add_plugin("python".to_string(), Box::new(plugin));
        log::info!("Python plugin installed");
    }
}

/* **************************************************************************************************************** */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */