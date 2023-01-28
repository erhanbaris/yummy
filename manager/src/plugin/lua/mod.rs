mod model;

#[cfg(test)]
mod test;

use std::path::Path;
use std::{rc::Rc, cell::RefCell, sync::Arc};
use std::fs;

use crate::auth::model::{DeviceIdAuthRequest, CustomIdAuthRequest, LogoutRequest, RefreshTokenRequest, RestoreTokenRequest};
use crate::plugin::YummyPlugin;

use general::config::YummyConfig;
use mlua::prelude::*;
use glob::glob;

use crate::plugin::EmailAuthRequest;

use super::YummyPluginInstaller;

macro_rules! create_func {
    ($pre: ident, $post: ident, $model: path) => {
        fn $pre <'a>(&self, model: Rc<RefCell<$model>>) -> anyhow::Result<()> { self.execute(model, stringify!($pre)) }
        fn $post <'a>(&self, model: Rc<RefCell<$model>>, successed: bool) -> anyhow::Result<()> { self.execute_with_result(model, successed, stringify!($post)) }    
    }
}

pub struct LuaPluginInstaller;

impl YummyPluginInstaller for LuaPluginInstaller {
    fn install(executer: super::PluginExecuter, config: Arc<YummyConfig>) -> super::PluginExecuter {
        let mut executer = executer;
        let mut plugin = LuaPlugin::new();

        let path = Path::new(&config.lua_files_path).join("*.lua").to_string_lossy().to_string();

        for path in glob(&path).expect("Failed to read glob pattern") {
            let content = fs::read_to_string(&path.unwrap().to_string_lossy().to_string()).unwrap();
            plugin.set_content(&content).unwrap();
        }
        
        if let Ok(paths) = fs::read_dir(&config.lua_files_path) {
            for path in paths {
                println!("Name: {}", path.unwrap().path().display())
            }
        }

        executer.add_plugin("lua".to_string(), Box::new(plugin));
        executer
    }
}

pub struct LuaPlugin {
    lua: Lua
}

impl LuaPlugin {
    pub fn new() -> Self {
        //let lua = unsafe { Lua::unsafe_new_with(LuaStdLib::ALL, LuaOptions::default()) };
        let lua = Lua::new();
        
        Self {
            lua,
        }
    }

    pub fn set_content(&mut self, content: &str) -> anyhow::Result<()> {
        self.lua.load(content, ).exec()?;
        Ok(())
    }

    fn execute<T: LuaUserData + 'static>(&self, model: Rc<RefCell<T>>, func_name: &str) -> anyhow::Result<()> {
        if let Ok(function) = self.lua.globals().get::<_, LuaFunction>(func_name) {
            function.call::<_, ()>(model)?;
            self.lua.gc_collect()?;
        }

        Ok(())
    }

    fn execute_with_result<T: LuaUserData + 'static>(&self, model: Rc<RefCell<T>>, successed: bool, func_name: &str) -> anyhow::Result<()> {
        if let Ok(function) = self.lua.globals().get::<_, LuaFunction>(func_name) {
            function.call::<_, ()>((model, successed))?;
            self.lua.gc_collect()?;
        }
        Ok(())
    }
}

impl YummyPlugin for LuaPlugin {
    create_func!(pre_email_auth, post_email_auth, EmailAuthRequest);
    create_func!(pre_deviceid_auth, post_deviceid_auth, DeviceIdAuthRequest);
    create_func!(pre_customid_auth, post_customid_auth, CustomIdAuthRequest);
    create_func!(pre_logout, post_logout, LogoutRequest);
    create_func!(pre_refresh_token, post_refresh_token, RefreshTokenRequest);
    create_func!(pre_restore_token, post_restore_token, RestoreTokenRequest);
}
