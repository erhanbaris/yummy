/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* **************************************************************************************************************** */
mod bind;
mod model;

#[cfg(test)]
mod test;

/* **************************************************************************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::path::Path;
use std::{rc::Rc, cell::RefCell, sync::Arc};
use std::fs;

use crate::auth::model::{DeviceIdAuthRequest, CustomIdAuthRequest, LogoutRequest, RefreshTokenRequest, RestoreTokenRequest, ConnUserDisconnect};
use crate::conn::model::UserConnected;
use crate::plugin::YummyPlugin;
use crate::room::model::{CreateRoomRequest, UpdateRoom, JoinToRoomRequest, ProcessWaitingUser, KickUserFromRoom, DisconnectFromRoomRequest, MessageToRoomRequest, RoomListRequest, WaitingRoomJoins, GetRoomRequest};
use crate::user::model::{GetUserInformation, UpdateUser};

use ::model::config::YummyConfig;
use mlua::prelude::*;
use glob::{MatchOptions, glob_with};

use crate::plugin::EmailAuthRequest;

use self::bind::buildin::inner_bind_buildin_functions;

use super::YummyPluginInstaller;

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* **************************************************** MACROS **************************************************** */
/* **************************************************************************************************************** */
macro_rules! create_func {
    ($pre: ident, $post: ident, $model: path) => {
        fn $pre <'a>(&self, model: Rc<RefCell<$model>>) -> anyhow::Result<()> { self.execute(model, stringify!($pre)) }
        fn $post <'a>(&self, model: Rc<RefCell<$model>>, successed: bool) -> anyhow::Result<()> { self.execute_with_result(model, successed, stringify!($post)) }    
    }
}

/* **************************************************************************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
#[derive(Default)]
pub struct LuaPluginInstaller;

pub struct LuaPlugin {
    lua: Lua
}

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* **************************************************************************************************************** */

#[allow(clippy::new_without_default)]
impl LuaPlugin {
    pub fn new() -> Self {
        //let lua = unsafe { Lua::unsafe_new_with(LuaStdLib::ALL, LuaOptions::default()) };
        let lua = Lua::new();
        Self { lua }
    }

    pub fn bind_buildin_functions(&mut self) -> LuaResult<()> {
        inner_bind_buildin_functions(&mut self.lua)
    }

    pub fn set_content(&mut self, content: &str) -> anyhow::Result<()> {
        self.lua.load(content, ).exec()?;
        Ok(())
    }

    fn execute<T: LuaUserData + 'static>(&self, model: Rc<RefCell<T>>, func_name: &str) -> anyhow::Result<()> {
        println!("Execute {}", func_name);
        if let Ok(function) = self.lua.globals().get::<_, LuaFunction>(func_name) {

            // Call related function from Lua scope
            function.call::<_, ()>(model)?;

            // Because of the Rc passed to Lua and we would like to get model from Rc but Lua keep the reference until next gc call
            self.lua.gc_collect()?;
        }

        Ok(())
    }

    fn execute_with_result<T: LuaUserData + 'static>(&self, model: Rc<RefCell<T>>, successed: bool, func_name: &str) -> anyhow::Result<()> {
        println!("Execute with result {}", func_name);
        if let Ok(function) = self.lua.globals().get::<_, LuaFunction>(func_name) {
            function.call::<_, ()>((model, successed))?;
            self.lua.gc_collect()?;
        }
        Ok(())
    }
}

/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
impl YummyPluginInstaller for LuaPluginInstaller {
    fn install(&self, executer: &mut super::PluginExecuter, config: Arc<YummyConfig>) {
        log::info!("Lua plugin installing");
        let mut plugin = LuaPlugin::new();

        let path = Path::new(&config.lua_files_path).join("*.lua").to_string_lossy().to_string();
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
                plugin.set_content(&content).unwrap();
                log::info!("Lua file imported: {}", path);
            }

            // Bind all in-build functions
            plugin.bind_buildin_functions().unwrap();

            // Bind all context related functions
            plugin.bind_context(executer).unwrap();
            
            executer.add_plugin("lua".to_string(), Box::new(plugin));
        }

        log::info!("Lua plugin installed");
    }
}

impl YummyPlugin for LuaPlugin {

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

/* **************************************************************************************************************** */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */
