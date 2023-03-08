/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* **************************************************************************************************************** */
#[cfg(test)]
mod test;
mod model;

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
use vm::convert::ToPyObject;
use vm::scope::Scope;
use vm::VirtualMachine;

use crate::{
    auth::model::{ConnUserDisconnect, CustomIdAuthRequest, DeviceIdAuthRequest, EmailAuthRequest, LogoutRequest, RefreshTokenRequest, RestoreTokenRequest},
    conn::model::UserConnected,
    room::model::{
        CreateRoomRequest, DisconnectFromRoomRequest, GetRoomRequest, JoinToRoomRequest, KickUserFromRoom, MessageToRoomRequest, ProcessWaitingUser, RoomListRequest, UpdateRoom, WaitingRoomJoins,
    },
    user::model::{GetUserInformation, UpdateUser},
};

use super::{YummyPlugin, YummyPluginInstaller};

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* **************************************************** MACROS **************************************************** */
/* **************************************************************************************************************** */
macro_rules! create_func {
    ($pre: ident, $post: ident, $target: path, $model: path) => {
        fn $pre<'a>(&self, _: Rc<RefCell<$model>>) -> anyhow::Result<()> {
            //let tmp_model = model.as_ref().clone();
            //self.execute(tmp_model, stringify!($pre), $target)
            Ok(())
        }
        fn $post<'a>(&self, _: Rc<RefCell<$model>>, _: bool) -> anyhow::Result<()> {
            //self.execute(model, stringify!($post), $target)
            Ok(())
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
    pub scopes: Vec<Scope>
}

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* **************************************************************************************************************** */
pub enum FunctionType {
    EMAIL_AUTH,
    DEVICEID_AUTH,
    CUSTOMID_AUTH,
    LOGOUT,
    REFRESH_TOKEN,
    RESTORE_TOKEN,
    USER_CONNECTED,
    USER_DISCONNECTED,
    GET_USER_INFORMATION,
    UPDATE_USER,
    CREATE_ROOM,
    UPDATE_ROOM,
    JOIN_TO_ROOM,
    PROCESS_WAITING_USER,
    KICK_USER_FROM_ROOM,
    DISCONNECT_FROM_ROOM_REQUEST,
    MESSAGE_TO_ROOM_REQUEST,
    ROOM_LIST_REQUEST,
    WAITING_ROOM_JOINS,
    GET_ROOM_REQUEST
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

/* **************************************************************************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* **************************************************************************************************************** */
impl PythonPlugin {
    pub fn execute<T: ToPyObject + 'static>(&self, model: T, name: &str, func_type: FunctionType) -> anyhow::Result<()> {
        self.interpreter.enter(|vm| {
            let mut model = vm.new_pyobj(model);
        
            for scope in self.scopes.iter() {
                let test_fn = scope.globals.get_item(name, vm).unwrap();
                model = match vm.invoke(&test_fn, (model,)) {
                    Ok(model) => model,
                    Err(error) => {
                        let mut error_message = String::new();
                        vm.write_exception(&mut error_message, &error).unwrap();
                        return Err(anyhow::anyhow!(error_message))
                    }
                };
            }

            Ok(())
        })
    }
}

impl PythonPluginInstaller {
    pub fn build_plugin(config: Arc<YummyConfig>) -> PythonPlugin {
        let mut scopes = Vec::new();

        let interpreter = vm::Interpreter::with_init(Default::default(), init_vm);
        interpreter
            .enter(|vm| -> vm::PyResult<()> {

                let path = Path::new(&config.python_files_path).join("*.py").to_string_lossy().to_string();
                log::info!("Searhing lua files at {}", path);
        
                let options = MatchOptions {
                    case_sensitive: false,
                    require_literal_separator: false,
                    require_literal_leading_dot: false,
                };
        
                let paths = glob_with(&path, options).unwrap();
                for path in paths {
                    let path = path.unwrap().to_string_lossy().to_string();
                    let content = fs::read_to_string(&path).unwrap();

                    let code_obj = vm.compile(&content, vm::compiler::Mode::Exec, "<embedded>".to_owned(),)
                        .map_err(|err| vm.new_syntax_error(&err))?;
                    
                    let scope = vm.new_scope_with_builtins();
                    
                    if let Err(error) = vm.run_code_obj(code_obj, scope.clone()) {
                        let mut error_message = String::new();
                        vm.write_exception(&mut error_message, &error).unwrap();
                        log::error!("'{}' failed to compile. Error message: {}", path, error_message);
                    }

                    scopes.push(scope);
                    
                    /*let test_fn = scope.globals.get_item("test", vm).unwrap();

                    let arg = vm.new_pyobj(123);

                    if let Err(error) = vm.invoke(&test_fn, (arg, )) {
                        let mut error_message = String::new();
                        vm.write_exception(&mut error_message, &error).unwrap();
                        log::error!("'{}' failed to compile. Error message: {}", path, error_message);
                    }*/
                }
                Ok(())
            })
            .unwrap();
        PythonPlugin { interpreter, scopes }
    }
}

/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
impl YummyPlugin for PythonPlugin {
    // Auth manager
    create_func!(pre_email_auth, post_email_auth, FunctionType::EMAIL_AUTH, EmailAuthRequest);
    create_func!(pre_deviceid_auth, post_deviceid_auth, FunctionType::DEVICEID_AUTH, DeviceIdAuthRequest);
    create_func!(pre_customid_auth, post_customid_auth, FunctionType::CUSTOMID_AUTH, CustomIdAuthRequest);
    create_func!(pre_logout, post_logout, FunctionType::LOGOUT, LogoutRequest);
    create_func!(pre_refresh_token, post_refresh_token, FunctionType::REFRESH_TOKEN, RefreshTokenRequest);
    create_func!(pre_restore_token, post_restore_token, FunctionType::RESTORE_TOKEN, RestoreTokenRequest);

    // Connection manager
    create_func!(pre_user_connected, post_user_connected, FunctionType::USER_CONNECTED, UserConnected);
    create_func!(pre_user_disconnected, post_user_disconnected, FunctionType::USER_DISCONNECTED, ConnUserDisconnect);

    // User manager
    create_func!(pre_get_user_information, post_get_user_information, FunctionType::GET_USER_INFORMATION, GetUserInformation);
    create_func!(pre_update_user, post_update_user, FunctionType::UPDATE_USER, UpdateUser);

    // Room Manager
    create_func!(pre_create_room, post_create_room, FunctionType::CREATE_ROOM, CreateRoomRequest);
    create_func!(pre_update_room, post_update_room, FunctionType::UPDATE_ROOM, UpdateRoom);
    create_func!(pre_join_to_room, post_join_to_room, FunctionType::JOIN_TO_ROOM, JoinToRoomRequest);
    create_func!(pre_process_waiting_user, post_process_waiting_user, FunctionType::PROCESS_WAITING_USER, ProcessWaitingUser);
    create_func!(pre_kick_user_from_room, post_kick_user_from_room, FunctionType::KICK_USER_FROM_ROOM, KickUserFromRoom);
    create_func!(pre_disconnect_from_room_request, post_disconnect_from_room_request, FunctionType::DISCONNECT_FROM_ROOM_REQUEST, DisconnectFromRoomRequest);
    create_func!(pre_message_to_room_request, post_message_to_room_request, FunctionType::MESSAGE_TO_ROOM_REQUEST, MessageToRoomRequest);
    create_func!(pre_room_list_request, post_room_list_request, FunctionType::ROOM_LIST_REQUEST, RoomListRequest);
    create_func!(pre_waiting_room_joins, post_waiting_room_joins, FunctionType::WAITING_ROOM_JOINS, WaitingRoomJoins);
    create_func!(pre_get_room_request, post_get_room_request, FunctionType::GET_ROOM_REQUEST, GetRoomRequest);
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
