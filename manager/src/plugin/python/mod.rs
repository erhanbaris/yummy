/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* **************************************************************************************************************** */
#[cfg(test)]
mod test;
mod model;

use std::collections::HashMap;
/* **************************************************************************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::{cell::RefCell, path::Path};
use std::rc::Rc;

use std::sync::Arc;
use std::fs;

use glob::{MatchOptions, glob_with};
use ::model::config::YummyConfig;
use rustpython_vm as vm;
use strum::IntoEnumIterator;
use vm::class::PyClassImpl;
use vm::convert::ToPyObject;
use vm::function::IntoFuncArgs;
use vm::scope::Scope;
use vm::{VirtualMachine, PyObjectRef};
use strum_macros::EnumIter;

use crate::plugin::python::model::DeviceIdAuthRequestWrapper;
use crate::{
    auth::model::{ConnUserDisconnect, CustomIdAuthRequest, DeviceIdAuthRequest, EmailAuthRequest, LogoutRequest, RefreshTokenRequest, RestoreTokenRequest},
    conn::model::UserConnected,
    room::model::{
        CreateRoomRequest, DisconnectFromRoomRequest, GetRoomRequest, JoinToRoomRequest, KickUserFromRoom, MessageToRoomRequest, ProcessWaitingUser, RoomListRequest, UpdateRoom, WaitingRoomJoins,
    },
    user::model::{GetUserInformation, UpdateUser},
};

use self::model::ModelWrapper;

use super::{YummyPlugin, YummyPluginInstaller};

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* **************************************************** MACROS **************************************************** */
/* **************************************************************************************************************** */
macro_rules! create_dummy_func {
    ($pre: ident, $post: ident, $target: path, $model: path) => {
        fn $pre<'a>(&self, _: Rc<RefCell<$model>>) -> anyhow::Result<()> {
            Ok(())
        }
        fn $post<'a>(&self, _: Rc<RefCell<$model>>, _: bool) -> anyhow::Result<()> {
            Ok(())
        }
    };
}

macro_rules! create_func {
    ($pre: ident, $post: ident, $function: path, $model: path, $wrapper: tt) => {
        fn $pre<'a>(&self, model: Rc<RefCell<$model>>) -> anyhow::Result<()> {
            self.execute_pre_functions::<_, $wrapper>(model, $function)
        }

        fn $post<'a>(&self, model: Rc<RefCell<$model>>, success: bool) -> anyhow::Result<()> {
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
    GetRoomRequest
}

/* **************************************************************************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* **************************************************************************************************************** */
fn init_vm(vm: &mut VirtualMachine) {
    vm.add_frozen(rustpython_pylib::frozen_stdlib());
}

/* **************************************************************************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* **************************************************************************************************************** */

/* ***************************************************************************************.clone()************************* */
/* ************************************************* IMPLEMENTS *************************************************** */
/* **************************************************************************************************************** */
impl PythonPlugin {
    pub fn execute_pre_functions<T, W: ToPyObject + rustpython_vm::PyPayload + ModelWrapper<Entity = T> + 'static>(&self, model: Rc<RefCell<T>>, function: FunctionType) -> anyhow::Result<()> {
        self.interpreter.enter(|vm| {
            let model = W::wrap(model).to_pyobject(vm);
            self.inner_execute(vm, &self.pre_function_refs, (model, ), function)
        })
    }

    pub fn execute_post_functions<T, W: ToPyObject + rustpython_vm::PyPayload + ModelWrapper<Entity = T> + 'static>(&self, model: Rc<RefCell<T>>, success: bool, function: FunctionType) -> anyhow::Result<()> {
        self.interpreter.enter(|vm| {
            let model = W::wrap(model).to_pyobject(vm);
            self.inner_execute(vm, &self.post_function_refs, (model, success), function)
        })
    }

    fn inner_execute(&self, vm: &VirtualMachine, functions: &HashMap<FunctionType, Vec<PyObjectRef>>, args: impl IntoFuncArgs + Clone, function: FunctionType) -> anyhow::Result<()> {

        // Get all parsed functions references to invoke
        if let Some(functions) = functions.get(&function) {

            // Iterate over the functions
            for function in functions.iter() {

                // Call the function with our model and check everyting went well
                if let Err(error) = vm.invoke(function, args.clone()) {

                    /*
                    We should return an error message instead of the exception object.
                    Our error message requires post-processing
                    */
                    let mut error_message = String::new();
                    vm.write_exception(&mut error_message, &error).unwrap();
                    return Err(anyhow::anyhow!(error_message))
                }
            }
        }

        Ok(())
    }
}

impl PythonPluginInstaller {
    pub fn build_plugin(config: Arc<YummyConfig>) -> PythonPlugin {
        let mut scopes = Vec::new();
        let mut pre_function_refs: HashMap<FunctionType, Vec<PyObjectRef>> = HashMap::new();
        let mut post_function_refs: HashMap<FunctionType, Vec<PyObjectRef>> = HashMap::new();

        let interpreter = vm::Interpreter::with_init(Default::default(), init_vm);
        interpreter
            .enter(|vm| -> vm::PyResult<()> {
                DeviceIdAuthRequestWrapper::make_class(&vm.ctx);

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
            })
            .unwrap();

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
            FunctionType::EmailAuth => "NOT_IMPLEMENTED_YET",
            FunctionType::DeviceidAuth => "pre_deviceid_auth",
            FunctionType::CustomidAuth => "NOT_IMPLEMENTED_YET",
            FunctionType::Logout => "NOT_IMPLEMENTED_YET",
            FunctionType::RefreshToken => "NOT_IMPLEMENTED_YET",
            FunctionType::RestoreToken => "NOT_IMPLEMENTED_YET",
            FunctionType::UserConnected => "NOT_IMPLEMENTED_YET",
            FunctionType::UserDisconnected => "NOT_IMPLEMENTED_YET",
            FunctionType::GetUserInformation => "NOT_IMPLEMENTED_YET",
            FunctionType::UpdateUser => "NOT_IMPLEMENTED_YET",
            FunctionType::CreateRoom => "NOT_IMPLEMENTED_YET",
            FunctionType::UpdateRoom => "NOT_IMPLEMENTED_YET",
            FunctionType::JoinToRoom => "NOT_IMPLEMENTED_YET",
            FunctionType::ProcessWaitingUser => "NOT_IMPLEMENTED_YET",
            FunctionType::KickUserFromRoom => "NOT_IMPLEMENTED_YET",
            FunctionType::DisconnectFromRoomRequest => "NOT_IMPLEMENTED_YET",
            FunctionType::MessageToRoomRequest => "NOT_IMPLEMENTED_YET",
            FunctionType::RoomListRequest => "NOT_IMPLEMENTED_YET",
            FunctionType::WaitingRoomJoins => "NOT_IMPLEMENTED_YET",
            FunctionType::GetRoomRequest => "NOT_IMPLEMENTED_YET",
        }
    }

    pub fn get_post_function_name(&self) -> &'static str {
        match self {
            FunctionType::EmailAuth => "NOT_IMPLEMENTED_YET",
            FunctionType::DeviceidAuth => "post_deviceid_auth",
            FunctionType::CustomidAuth => "NOT_IMPLEMENTED_YET",
            FunctionType::Logout => "NOT_IMPLEMENTED_YET",
            FunctionType::RefreshToken => "NOT_IMPLEMENTED_YET",
            FunctionType::RestoreToken => "NOT_IMPLEMENTED_YET",
            FunctionType::UserConnected => "NOT_IMPLEMENTED_YET",
            FunctionType::UserDisconnected => "NOT_IMPLEMENTED_YET",
            FunctionType::GetUserInformation => "NOT_IMPLEMENTED_YET",
            FunctionType::UpdateUser => "NOT_IMPLEMENTED_YET",
            FunctionType::CreateRoom => "NOT_IMPLEMENTED_YET",
            FunctionType::UpdateRoom => "NOT_IMPLEMENTED_YET",
            FunctionType::JoinToRoom => "NOT_IMPLEMENTED_YET",
            FunctionType::ProcessWaitingUser => "NOT_IMPLEMENTED_YET",
            FunctionType::KickUserFromRoom => "NOT_IMPLEMENTED_YET",
            FunctionType::DisconnectFromRoomRequest => "NOT_IMPLEMENTED_YET",
            FunctionType::MessageToRoomRequest => "NOT_IMPLEMENTED_YET",
            FunctionType::RoomListRequest => "NOT_IMPLEMENTED_YET",
            FunctionType::WaitingRoomJoins => "NOT_IMPLEMENTED_YET",
            FunctionType::GetRoomRequest => "NOT_IMPLEMENTED_YET",
        }
    }
}

/* **************************************************************************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* **************************************************************************************************************** */
impl YummyPlugin for PythonPlugin {
    // Auth manager
    create_dummy_func!(pre_email_auth, post_email_auth, FunctionType::EMAIL_AUTH, EmailAuthRequest);
    create_func!(pre_deviceid_auth, post_deviceid_auth, FunctionType::DeviceidAuth, DeviceIdAuthRequest, DeviceIdAuthRequestWrapper);
    create_dummy_func!(pre_customid_auth, post_customid_auth, FunctionType::CUSTOMID_AUTH, CustomIdAuthRequest);
    create_dummy_func!(pre_logout, post_logout, FunctionType::LOGOUT, LogoutRequest);
    create_dummy_func!(pre_refresh_token, post_refresh_token, FunctionType::REFRESH_TOKEN, RefreshTokenRequest);
    create_dummy_func!(pre_restore_token, post_restore_token, FunctionType::RESTORE_TOKEN, RestoreTokenRequest);

    // Connection manager
    create_dummy_func!(pre_user_connected, post_user_connected, FunctionType::USER_CONNECTED, UserConnected);
    create_dummy_func!(pre_user_disconnected, post_user_disconnected, FunctionType::USER_DISCONNECTED, ConnUserDisconnect);

    // User manager
    create_dummy_func!(pre_get_user_information, post_get_user_information, FunctionType::GET_USER_INFORMATION, GetUserInformation);
    create_dummy_func!(pre_update_user, post_update_user, FunctionType::UPDATE_USER, UpdateUser);

    // Room Manager
    create_dummy_func!(pre_create_room, post_create_room, FunctionType::CREATE_ROOM, CreateRoomRequest);
    create_dummy_func!(pre_update_room, post_update_room, FunctionType::UPDATE_ROOM, UpdateRoom);
    create_dummy_func!(pre_join_to_room, post_join_to_room, FunctionType::JOIN_TO_ROOM, JoinToRoomRequest);
    create_dummy_func!(pre_process_waiting_user, post_process_waiting_user, FunctionType::PROCESS_WAITING_USER, ProcessWaitingUser);
    create_dummy_func!(pre_kick_user_from_room, post_kick_user_from_room, FunctionType::KICK_USER_FROM_ROOM, KickUserFromRoom);
    create_dummy_func!(pre_disconnect_from_room_request, post_disconnect_from_room_request, FunctionType::DISCONNECT_FROM_ROOM_REQUEST, DisconnectFromRoomRequest);
    create_dummy_func!(pre_message_to_room_request, post_message_to_room_request, FunctionType::MESSAGE_TO_ROOM_REQUEST, MessageToRoomRequest);
    create_dummy_func!(pre_room_list_request, post_room_list_request, FunctionType::ROOM_LIST_REQUEST, RoomListRequest);
    create_dummy_func!(pre_waiting_room_joins, post_waiting_room_joins, FunctionType::WAITING_ROOM_JOINS, WaitingRoomJoins);
    create_dummy_func!(pre_get_room_request, post_get_room_request, FunctionType::GET_ROOM_REQUEST, GetRoomRequest);
}

impl YummyPluginInstaller for PythonPluginInstaller {
    fn install(&self, executer: &mut super::PluginExecuter, config: Arc<YummyConfig>) {
        log::info!("Python plugin installing");

        let plugin = PythonPluginInstaller::build_plugin(config);
        executer.add_plugin("python".to_string(), Box::new(plugin));
        log::info!("Python plugin installed");
    }
}

/* **************************************************************************************************************** */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */
