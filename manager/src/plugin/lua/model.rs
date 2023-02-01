use std::ops::Deref;

use general::{password::Password, model::UserId};

use mlua::prelude::*;

use crate::{auth::model::{EmailAuthRequest, DeviceIdAuthRequest, CustomIdAuthRequest, LogoutRequest, RefreshTokenRequest, RestoreTokenRequest, ConnUserDisconnect}, conn::model::UserConnected, user::model::{GetUserInformation, GetUserInformationEnum, UpdateUser}};

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

impl LuaUserData for GetUserInformation {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_query", |_, this, ()| {
            Ok(this.query.clone())  
        });
    }
}

impl LuaUserData for GetUserInformationEnum {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {

        methods.add_method("get_type", |_, this, ()| {
            match this {
                GetUserInformationEnum::Me(_) => Ok("Me"),
                GetUserInformationEnum::User { user: _, requester: _ } => Ok("User"),
                GetUserInformationEnum::UserViaSystem(_) => Ok("UserViaSystem"),
            }
        });


        methods.add_method("as_table", |lua, this, ()| {
            let mut v = LuaMultiValue::new();
        
            let array_table = match this {
                GetUserInformationEnum::Me(me) => {
                    let array_table = lua.create_table()?;
                    array_table.set("type", "Me")?;
                    match me.deref() {
                        Some(user) => {
                            array_table.set("user_id", user.user.to_string())?;
                            array_table.set("session_id", user.session.to_string())?;
                        }
                        None => {
                            array_table.set("user_id", "")?;
                            array_table.set("session_id", "")?;
                        }
                    }
                    array_table
                },
                GetUserInformationEnum::User { user, requester } => {
                    let array_table = lua.create_table()?;
                    array_table.set("type", "UserViaSystem")?;
                    array_table.set("user_id", user.to_string())?;
                    match requester.deref() {
                        Some(user) => {
                            array_table.set("requester_user_id", user.user.to_string())?;
                            array_table.set("requester_session_id", user.session.to_string())?;
                        }
                        None => {
                            array_table.set("requester_user_id", "")?;
                            array_table.set("requester_session_id", "")?;
                        }
                    }
                    array_table
                },
                GetUserInformationEnum::UserViaSystem(user_id) => {
                    let array_table = lua.create_table()?;
                    array_table.set("type", "UserViaSystem")?;
                    array_table.set("user_id", user_id.to_string())?;
                    array_table
                }
            };
            v.push_front(mlua::Value::Table(array_table));
            Ok(v)
        });
    }
}

impl LuaUserData for UpdateUser {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        auth_macros!(methods);
        methods.add_method("get_target_user_id", |_, this, ()| Ok(this.target_user_id.as_ref().map(|item| item.to_string()).unwrap_or_default()));
        methods.add_method_mut("set_target_user_id", |_, this, target_user_id: String| {
            if target_user_id.is_empty()  {
                this.target_user_id = None;
            } else {
                this.target_user_id = Some(UserId::from(target_user_id));
            }
            
            Ok(())
        });
    }
}