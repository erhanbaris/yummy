use std::sync::Arc;
use local_impl::local_impl;

use interface::{auth::{YummyAuthInterface, YummyEmailAuthModel}, UserProxy};

use mlua::prelude::*;

pub struct LuaYummyAuthPlugin {
    lua: Lua,
    pre_email_auth_func: bool
}

impl LuaYummyAuthPlugin {
    pub fn new() -> Self {
        let lua = Lua::new();

        {
            let pre_email_auth_func: LuaFunction = lua.load(r#"
                function(message)
                    print("Lua is alive")
                    print(message)
                end
            "#,).eval().unwrap();

            let globals = lua.globals();
            globals.set("pre_email", pre_email_auth_func).unwrap();
        }

        Self {
            lua,
            pre_email_auth_func: true
        }
    }
}

impl YummyAuthInterface for LuaYummyAuthPlugin {
    fn pre_email_auth<'a>(&self, user_manager: &'a dyn UserProxy, model: &mut YummyEmailAuthModel) -> anyhow::Result<()> {
        let YummyEmailAuthModel { ref_id, auth, email, password, if_not_exist_create, socket } = model;
        if self.pre_email_auth_func {
            let func: LuaFunction = self.lua.globals().get("pre_email").unwrap();
            func.call::<_, ()>(&email[..]).unwrap();
        }

        Ok(())
    }

    fn post_email_auth<'a>(&self, user_manager: &'a dyn UserProxy, model: &mut YummyEmailAuthModel) -> anyhow::Result<()> {
        Ok(())
    }
}

#[local_impl]
impl LuaUserData for YummyEmailAuthModel {
}