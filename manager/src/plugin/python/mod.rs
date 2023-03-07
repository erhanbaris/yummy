/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* **************************************************************************************************************** */

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
    ($pre: ident, $post: ident, $model: path) => {
        fn $pre<'a>(&self, model: Rc<RefCell<$model>>) -> anyhow::Result<()> {
            Ok(())
        }
        fn $post<'a>(&self, model: Rc<RefCell<$model>>, successed: bool) -> anyhow::Result<()> {
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
}

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* **************************************************************************************************************** */

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

/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
impl YummyPlugin for PythonPlugin {
    // Auth manager
    create_func!(pre_email_auth, post_email_auth, EmailAuthRequest);
    create_func!(pre_deviceid_auth, post_deviceid_auth, DeviceIdAuthRequest);
    create_func!(pre_customid_auth, post_customid_auth, CustomIdAuthRequest);
    create_func!(pre_logout, post_logout, LogoutRequest);
    create_func!(pre_refresh_token, post_refresh_token, RefreshTokenRequest);
    create_func!(pre_restore_token, post_restore_token, RestoreTokenRequest);

    // Connection manager
    create_func!(pre_user_connected, post_user_connected, UserConnected);
    create_func!(pre_user_disconnected, post_user_disconnected, ConnUserDisconnect);

    // User manager
    create_func!(pre_get_user_information, post_get_user_information, GetUserInformation);
    create_func!(pre_update_user, post_update_user, UpdateUser);

    // Room Manager
    create_func!(pre_create_room, post_create_room, CreateRoomRequest);
    create_func!(pre_update_room, post_update_room, UpdateRoom);
    create_func!(pre_join_to_room, post_join_to_room, JoinToRoomRequest);
    create_func!(pre_process_waiting_user, post_process_waiting_user, ProcessWaitingUser);
    create_func!(pre_kick_user_from_room, post_kick_user_from_room, KickUserFromRoom);
    create_func!(pre_disconnect_from_room_request, post_disconnect_from_room_request, DisconnectFromRoomRequest);
    create_func!(pre_message_to_room_request, post_message_to_room_request, MessageToRoomRequest);
    create_func!(pre_room_list_request, post_room_list_request, RoomListRequest);
    create_func!(pre_waiting_room_joins, post_waiting_room_joins, WaitingRoomJoins);
    create_func!(pre_get_room_request, post_get_room_request, GetRoomRequest);
}

impl YummyPluginInstaller for PythonPluginInstaller {
    fn install(&self, executer: &mut super::PluginExecuter, config: Arc<YummyConfig>) {
        log::info!("Python plugin installing");
        let interpreter = vm::Interpreter::with_init(Default::default(), init_vm);
        let mut file_objects = Vec::new();

        interpreter
            .enter(|vm| -> vm::PyResult<()> {

                let path = Path::new(&config.python_files_path).join("*.py").to_string_lossy().to_string();
                log::info!("Searhing lua files at {}", path);
        
                let options = MatchOptions {
                    case_sensitive: false,
                    require_literal_separator: false,
                    require_literal_leading_dot: false,
                };
        
                if let Ok(paths) = glob_with(&path, options) {
                    for path in paths {
                        let path = path.unwrap().to_string_lossy().to_string();
                        let content = fs::read_to_string(&path).unwrap();

                        let code_obj = vm.compile(&content, vm::compiler::Mode::Exec, "<embedded>".to_owned(),)
                            .map_err(|err| vm.new_syntax_error(&err))?;
                        let file_object = vm.run_code_obj(code_obj, vm.new_scope_with_builtins()).unwrap();
                        file_objects.push(file_object);
                    }
                }

                //vm.call_method(&a, "", ());
                println!("in vm");
                Ok(())
            })
            .unwrap();
        executer.add_plugin("python".to_string(), Box::new(PythonPlugin { interpreter }));
        log::info!("Python plugin installed");
    }
}

/* **************************************************************************************************************** */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */
