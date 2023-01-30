use std::ops::Deref;

use general::password::Password;

use mlua::prelude::*;

use crate::{auth::model::{EmailAuthRequest, DeviceIdAuthRequest, CustomIdAuthRequest, LogoutRequest, RefreshTokenRequest, RestoreTokenRequest, ConnUserDisconnect}, conn::model::UserConnected};

macro_rules! auth_macros {
    ($methods: expr) => {
        $methods.add_method("get_user_id", |_, this, ()| {
            let user_id = match this.auth.deref() {
                Some(auth) => auth.user.to_string(),
                None => String::new()
            };
            Ok(user_id)
        });
        $methods.add_method("get_session_id", |_, this, ()| {
            let session_id = match this.auth.deref() {
                Some(auth) => auth.session.to_string(),
                None => String::new()
            };
            Ok(session_id)
        });
    }
}

impl LuaUserData for EmailAuthRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        auth_macros!(methods);
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

impl LuaUserData for DeviceIdAuthRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        auth_macros!(methods);
        methods.add_method("get_id", |_, this, ()| Ok(this.id.clone()));
        methods.add_method_mut("set_id", |_, this, id: String| {
            this.id = id;
            Ok(())
        });
    }
}

impl LuaUserData for CustomIdAuthRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        auth_macros!(methods);
        methods.add_method("get_id", |_, this, ()| Ok(this.id.clone()));
        methods.add_method_mut("set_id", |_, this, id: String| {
            this.id = id;
            Ok(())
        });
    }
}

impl LuaUserData for LogoutRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        auth_macros!(methods);
    }
}

impl LuaUserData for RefreshTokenRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        auth_macros!(methods);
        methods.add_method("get_token", |_, this, ()| Ok(this.token.clone()));
        methods.add_method_mut("set_token", |_, this, token: String| {
            this.token = token;
            Ok(())
        });
    }
}

impl LuaUserData for RestoreTokenRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        auth_macros!(methods);
        methods.add_method("get_token", |_, this, ()| Ok(this.token.clone()));
        methods.add_method_mut("set_token", |_, this, token: String| {
            this.token = token;
            Ok(())
        });
    }
}

impl LuaUserData for UserConnected {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_user_id", |_, this, ()| Ok(this.user_id.to_string()));
    }
}

impl LuaUserData for ConnUserDisconnect {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        auth_macros!(methods);
        methods.add_method("get_send_message", |_, this, ()| Ok(this.send_message));
        methods.add_method_mut("set_send_message", |_, this, send_message: bool| {
            this.send_message = send_message;
            Ok(())
        });
    }
}
