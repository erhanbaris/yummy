mod model;

#[cfg(test)]
mod test;

use std::path::Path;
use std::{rc::Rc, cell::RefCell, sync::Arc};
use std::fs;

use crate::auth::model::{DeviceIdAuthRequest, CustomIdAuthRequest, LogoutRequest, RefreshTokenRequest, RestoreTokenRequest, ConnUserDisconnect};
use crate::conn::model::UserConnected;
use crate::plugin::YummyPlugin;
use crate::user::model::{GetUserInformation, UpdateUser};

use general::config::YummyConfig;
use general::meta::{UserMetaAccess, MetaType};
use mlua::prelude::*;
use glob::{MatchOptions, glob_with};

use crate::plugin::EmailAuthRequest;

use super::YummyPluginInstaller;

macro_rules! create_func {
    ($pre: ident, $post: ident, $model: path) => {
        fn $pre <'a>(&self, model: Rc<RefCell<$model>>) -> anyhow::Result<()> { self.execute(model, stringify!($pre)) }
        fn $post <'a>(&self, model: Rc<RefCell<$model>>, successed: bool) -> anyhow::Result<()> { self.execute_with_result(model, successed, stringify!($post)) }    
    }
}

#[derive(Default)]
pub struct LuaPluginInstaller;

impl YummyPluginInstaller for LuaPluginInstaller {
    fn install(&self, executer: &mut super::PluginExecuter, config: Arc<YummyConfig>) {
        println!("Lua plugin installing");
        let mut plugin = LuaPlugin::new();

        let path = Path::new(&config.lua_files_path).join("*.lua").to_string_lossy().to_string();
        println!("Searhing lua files at {}", path);

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
                println!("Lua file imported: {}", path);
            }

            // Bind all in-build functions
            plugin.bind_buildin_functions().unwrap();
            
            executer.add_plugin("lua".to_string(), Box::new(plugin));
        }

        println!("Lua plugin installed");
    }
}

fn new_user_meta<'a>(lua: &'a Lua, (value, access): (LuaValue, UserMetaAccess)) -> LuaResult<LuaValue<'a>> {
    let value = match value {
        LuaValue::Nil => Ok(MetaType::Null),
        LuaValue::Boolean(val) => Ok(MetaType::Bool(val, access)),
        LuaValue::LightUserData(_) => Ok(MetaType::Null),
        LuaValue::Integer(val) => Ok(MetaType::Number(val as f64, access)),
        LuaValue::Number(val) => Ok(MetaType::Number(val, access)),
        LuaValue::String(val) =>  Ok(MetaType::String(val.to_str().unwrap_or_default().to_string(), access)),
        LuaValue::Table(table) => {
            let mut array = Vec::new();
            
            for row in table.sequence_values::<MetaType<UserMetaAccess>>() {
                let row = row?;
                array.push(row);
            }

            Ok(MetaType::List(Box::new(array), access))
        },
        LuaValue::Function(_) => Err(mlua::Error::RuntimeError("Meta does not have support for 'Function' type.".to_string())),
        LuaValue::Thread(_) => Err(mlua::Error::RuntimeError("Meta does not have support for 'Thread' type.".to_string())),
        LuaValue::UserData(_) => Err(mlua::Error::RuntimeError("Meta does not have support for 'UserData' type.".to_string())),
        LuaValue::Error(_) => Err(mlua::Error::RuntimeError("Meta does not have support for 'Error' type.".to_string())),
    };

    let value = value?;

    let value = lua.create_userdata(value)?;
    Ok(LuaValue::UserData(value))
}

pub struct LuaPlugin {
    lua: Lua
}
#[allow(clippy::new_without_default)]
impl LuaPlugin {
    pub fn new() -> Self {
        //let lua = unsafe { Lua::unsafe_new_with(LuaStdLib::ALL, LuaOptions::default()) };
        let lua = Lua::new();
        Self { lua }
    }

    pub fn bind_buildin_functions(&mut self) -> LuaResult<()> {
        let globals = self.lua.globals();
        globals.set("new_user_meta", self.lua.create_function(new_user_meta)?)?;
        Ok(())
    }

    pub fn set_content(&mut self, content: &str) -> anyhow::Result<()> {
        self.lua.load(content, ).exec()?;
        Ok(())
    }

    fn execute<T: LuaUserData + 'static>(&self, model: Rc<RefCell<T>>, func_name: &str) -> anyhow::Result<()> {
        println!("Execute {}", func_name);
        if let Ok(function) = self.lua.globals().get::<_, LuaFunction>(func_name) {
            function.call::<_, ()>(model)?;
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
}
