use std::ops::Deref;

use general::password::Password;
use crate::auth::YummyEmailAuthModel;

use mlua::prelude::*;

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum CallbackType {
    PreEmailAuth = 1,
    PostEmailAuth = 2
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
