mod model;

#[cfg(test)]
mod test;

use std::{rc::Rc, cell::RefCell, collections::HashSet};

use crate::plugin::auth::YummyAuthInterface;

use mlua::prelude::*;
use mlua::ExternalError;

use self::model::CallbackType;
use crate::plugin::EmailAuthRequest;

pub struct LuaYummyAuthPlugin {
    lua: Lua,
    callbacks: HashSet<CallbackType>
}

impl LuaYummyAuthPlugin {
    pub fn new() -> Self {
        //let lua = unsafe { Lua::unsafe_new_with(LuaStdLib::ALL, LuaOptions::default()) };
        let lua = Lua::new();
        
        Self {
            lua,
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

    fn execute_with_result<T: LuaUserData + 'static>(&self, model: Rc<RefCell<T>>, successed: bool, callback_type: CallbackType) -> anyhow::Result<()> {
        if !self.callbacks.contains(&callback_type) {
            return Ok(())
        }

        let func: LuaFunction = self.lua.globals().get(callback_type as u8)?;
        func.call::<_, ()>((model, successed))?;
        self.lua.gc_collect()?;
        Ok(())
    }
}

impl YummyAuthInterface for LuaYummyAuthPlugin {
    fn pre_email_auth<'a>(&self, model: Rc<RefCell<EmailAuthRequest>>) -> anyhow::Result<()> {
        self.execute(model, CallbackType::PreEmailAuth)
    }

    fn post_email_auth<'a>(&self, model: Rc<RefCell<EmailAuthRequest>>, successed: bool) -> anyhow::Result<()> {
        self.execute_with_result(model, successed, CallbackType::PostEmailAuth)
    }
}
