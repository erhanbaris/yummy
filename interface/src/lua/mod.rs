use std::{rc::Rc, cell::RefCell, ops::Deref};

use general::password::Password;
use crate::{auth::{YummyAuthInterface, YummyEmailAuthModel}, UserProxy};

use mlua::prelude::*;

pub struct LuaYummyAuthPlugin {
    lua: Lua,
    pre_email_auth_func: bool,
    post_email_auth_func: bool
}

impl LuaYummyAuthPlugin {
    pub fn new() -> Self {
        let lua = Lua::new();
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

            let globals = lua.globals();
            globals.set("pre_email", pre_email_auth_func).unwrap();


            let post_email_auth_func: LuaFunction = lua.load(r#"
                function(message)
                    print("post_email_auth_func")
                    print(message:get_session_id())
                    print(message:get_user_id())
                end
            "#,).eval().unwrap();

            let globals = lua.globals();
            globals.set("post_email", post_email_auth_func).unwrap();
        }

        Self {
            lua,
            pre_email_auth_func: true,
            post_email_auth_func: true
        }
    }
}

impl YummyAuthInterface for LuaYummyAuthPlugin {
    fn pre_email_auth<'a>(&self, _user_manager: &'a dyn UserProxy, model: Rc<RefCell<YummyEmailAuthModel>>) -> anyhow::Result<()> {
        if self.pre_email_auth_func {
            let func: LuaFunction = self.lua.globals().get("pre_email").unwrap();
            func.call::<_, ()>(model).unwrap();
            self.lua.gc_collect().unwrap();
        }

        Ok(())
    }

    fn post_email_auth<'a>(&self, _user_manager: &'a dyn UserProxy, model: Rc<RefCell<YummyEmailAuthModel>>) -> anyhow::Result<()> {
        if self.post_email_auth_func {
            let func: LuaFunction = self.lua.globals().get("post_email").unwrap();
            func.call::<_, ()>(model).unwrap();
            self.lua.gc_collect().unwrap();
        }

        Ok(())
    }
}

impl LuaUserData for YummyEmailAuthModel {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_ref_id", |_, this, ()| Ok(this.ref_id));
        methods.add_method("get_user_id", |_, this, ()| {
            let user_id = match this.auth.deref() {
                Some(auth) => auth.user.to_string(),
                None => String::new()
            };
            Ok(user_id)
        });
        methods.add_method("get_session_id", |_, this, ()| {
            let session_id = match this.auth.deref() {
                Some(auth) => auth.session.to_string(),
                None => String::new()
            };
            Ok(session_id)
        });
        methods.add_method("get_email", |_, this, ()| Ok(this.email.clone()));
        methods.add_method("get_password", |_, this, ()| Ok(this.password.get().clone()));
        methods.add_method("get_create", |_, this, ()| Ok(this.if_not_exist_create));

        methods.add_method_mut("set_email", |_, this, email: String| {
            this.email = email;
            Ok(())
        });
        methods.add_method_mut("set_password", |_, this, password: String| {
            this.password = Password::from(password);
            Ok(())
        });
        methods.add_method_mut("set_create", |_, this, create: bool| {
            this.if_not_exist_create = create;
            Ok(())
        });
    }
}
