mod model;

#[cfg(test)]
mod test;

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
        Self {
            lua: Lua::new(),
            callbacks: HashSet::default()
        }
    }

    pub fn add(&mut self, callback_type: CallbackType, content: &str) -> anyhow::Result<()> {
        let func: LuaFunction = self.lua.load(content, ).eval()?;
        self.callbacks.insert(callback_type);

        let globals = self.lua.globals();
        globals.set(callback_type as u8, func)?;
        Ok(())
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
