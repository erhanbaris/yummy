mod model;

use std::{rc::Rc, cell::RefCell, collections::HashSet};

use crate::{auth::{YummyAuthInterface, YummyEmailAuthModel}, UserProxy};

use mlua::prelude::*;

use self::model::CallbackType;

pub struct LuaYummyAuthPlugin {
    lua: Lua,
    callbacks: HashSet<CallbackType>
}

impl LuaYummyAuthPlugin {
    pub fn new() -> Self {
        let lua = Lua::new();
        let mut callbacks: HashSet<CallbackType> = HashSet::default();
        {
            let pre_email_auth_func: LuaFunction = lua.load(r#"
                function(message)
                    print("pre_email_auth_func")
                    print(message:get_email())
                    message:set_email("erhan@erhan.com")
                    print(message:get_email())
                    print(message:get_session_id())
                    print(message:get_user_id())
                end
            "#,).eval().unwrap();

            callbacks.insert(CallbackType::PreEmailAuth);

            let globals = lua.globals();
            globals.set("pre_email", pre_email_auth_func).unwrap();


            let post_email_auth_func: LuaFunction = lua.load(r#"
                function(message)
                    print("post_email_auth_func")
                    print(message:get_session_id())
                    print(message:get_user_id())
                end
            "#,).eval().unwrap();

            callbacks.insert(CallbackType::PostEmailAuth);

            let globals = lua.globals();
            globals.set("post_email", post_email_auth_func).unwrap();
        }

        Self {
            lua,
            callbacks: HashSet::default()
        }
    }

    fn execute<T: LuaUserData + 'static>(&self, model: Rc<RefCell<T>>, callback_type: CallbackType) -> anyhow::Result<()> {
        if !self.callbacks.contains(&callback_type) {
            return Ok(())
        }

        let func: LuaFunction = self.lua.globals().get(callback_type as u8)?;
        func.call::<_, ()>(model)?;
        self.lua.gc_collect()?;
        Ok(())
    }
}

impl YummyAuthInterface for LuaYummyAuthPlugin {
    fn pre_email_auth<'a>(&self, _user_manager: &'a dyn UserProxy, model: Rc<RefCell<YummyEmailAuthModel>>) -> anyhow::Result<()> {
        self.execute(model, CallbackType::PreEmailAuth)
    }

    fn post_email_auth<'a>(&self, _user_manager: &'a dyn UserProxy, model: Rc<RefCell<YummyEmailAuthModel>>) -> anyhow::Result<()> {
        self.execute(model, CallbackType::PostEmailAuth)
    }
}
