/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::{ops::Deref, collections::HashMap};

use general::password::Password;
use model::{UserId, UserType, CreateRoomAccessType, meta::{MetaAction, UserMetaAccess}};
use model::meta::RoomMetaAccess;
use cache::state::RoomInfoTypeVariant;
use lua::{CreateRoomAccessTypeWrapper, RoomUserTypeWrapper, UserIdWrapper, RoomInfoTypeVariantWrapper, RoomIdWrapper, MetaActionWrapper, UserTypeWrapper, UserMetaAccessWrapper, RoomMetaAccessWrapper};

use lua::MetaTypeWrapper;
use mlua::prelude::*;

use crate::{auth::model::{EmailAuthRequest, DeviceIdAuthRequest, CustomIdAuthRequest, LogoutRequest, RefreshTokenRequest, RestoreTokenRequest, ConnUserDisconnect}, conn::model::UserConnected, user::model::{GetUserInformation, GetUserInformationEnum, UpdateUser}, room::model::{CreateRoomRequest, UpdateRoom, JoinToRoomRequest, ProcessWaitingUser, KickUserFromRoom, DisconnectFromRoomRequest, MessageToRoomRequest, RoomListRequest, WaitingRoomJoins, GetRoomRequest}};

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* **************************************************** MACROS **************************************************** */
/* **************************************************************************************************************** */
macro_rules! general_macros {
    ($methods: expr) => {
        $methods.add_method("get_request_id", |_, this, ()| {
            let request_id = match this.request_id {
                Some(request_id) => request_id.clone(),
                None => 0
            };
            Ok(request_id)
        });
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

macro_rules! nullable_set {
    ($methods: expr, $name: expr, $field_name: ident, $field_type: ty) => {
        nullable_set!($methods, String, $name, $field_name, $field_type);
    };

    ($methods: expr, $lua_type: ty, $name: expr, $field_name: ident, $field_type: ty) => {
        $methods.add_method_mut($name, |_, this, field: Option<$lua_type>| {
            let result = field.map(|id| <$field_type>::try_from(id));
            match result {
                Some(Ok(field)) => {
                    this.$field_name = Some(field);
                    Ok(())
                },
                Some(Err(error)) => Err(mlua::Error::RuntimeError(error.to_string())),
                None => {
                    this.$field_name = None;
                    Ok(())
                }
            }
        });
    }
}

macro_rules! nullable_get {
    ($methods: expr, $name: expr, $field_name: ident, $wrapper: ident) => {
        $methods.add_method($name, |_, this, ()| {
            match this.$field_name.clone() {
                Some(item) => Ok(Some($wrapper(item))),
                None => Ok(None)
            }
        });
    };
}

macro_rules! get {
    ($methods: expr, $name: expr, $field_name: ident) => {
        $methods.add_method($name, |_, this, ()| Ok(this.$field_name.clone()));
    };
}

macro_rules! get_wrapper {
    ($methods: expr, $name: expr, $field_name: ident, $wrapper: ident) => {
        $methods.add_method($name, |_, this, ()| Ok($wrapper(this.$field_name.clone())));
    };
}

macro_rules! set {
    ($methods: expr, $name: expr, $field_name: ident, $field_type: ty) => {
        $methods.add_method_mut($name, |_, this, $field_name: $field_type| {
            this.$field_name = $field_name;
            Ok(())
        });
    };
}

macro_rules! set_wrapper {
    ($methods: expr, $name: expr, $field_name: ident, $field_type: ty) => {
        $methods.add_method_mut($name, |_, this, $field_name: $field_type| {
            this.$field_name = $field_name.0;
            Ok(())
        });
    };
}

/* **************************************************************************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* **************************************************************************************************************** */
impl LuaUserData for EmailAuthRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        general_macros!(methods);
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
        general_macros!(methods);
        methods.add_method("get_id", |_, this, ()| Ok(this.id.clone()));
        methods.add_method_mut("set_id", |_, this, id: String| {
            this.id = id;
            Ok(())
        });
    }
}

impl LuaUserData for CustomIdAuthRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        general_macros!(methods);
        methods.add_method("get_id", |_, this, ()| Ok(this.id.clone()));
        methods.add_method_mut("set_id", |_, this, id: String| {
            this.id = id;
            Ok(())
        });
    }
}

impl LuaUserData for LogoutRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        general_macros!(methods);
    }
}

impl LuaUserData for RefreshTokenRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        general_macros!(methods);
        methods.add_method("get_token", |_, this, ()| Ok(this.token.clone()));
        methods.add_method_mut("set_token", |_, this, token: String| {
            this.token = token;
            Ok(())
        });
    }
}

impl LuaUserData for RestoreTokenRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        general_macros!(methods);
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
        general_macros!(methods);
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
        general_macros!(methods);
        methods.add_method("get_target_user_id", |_, this, ()| Ok(this.target_user_id.as_ref().map(|item| item.to_string())));
        methods.add_method("get_metas", |_, this, ()| {
            Ok(this.metas.clone().map(|meta| meta.into_iter().map(|(key, value)| (key, MetaTypeWrapper(value))).collect::<HashMap<String, MetaTypeWrapper<UserMetaAccess>>>()))
        });
        get!(methods, "get_name", name);
        get!(methods, "get_email", email);
        get!(methods, "get_password", password);
        get!(methods, "get_device_id", device_id);
        get!(methods, "get_custom_id", custom_id);
        nullable_get!(methods, "get_user_type", user_type, UserTypeWrapper);
        nullable_get!(methods, "get_meta_action", meta_action, MetaActionWrapper);

        nullable_set!(methods, "set_target_user_id", target_user_id, UserId);
        nullable_set!(methods, "set_name", name, String);
        nullable_set!(methods, "set_email", email, String);
        nullable_set!(methods, "set_password", password, String);
        nullable_set!(methods, "set_device_id", device_id, String);
        nullable_set!(methods, "set_custom_id", custom_id, String);
        nullable_set!(methods, i32, "set_user_type", user_type, UserType);
        nullable_set!(methods, i32, "set_meta_action", meta_action, MetaAction);
        
        methods.add_method_mut("set_metas", |_, this, metas: Option<HashMap<String, MetaTypeWrapper<UserMetaAccessWrapper>>>| {
            this.metas = metas.map(|meta| meta.into_iter().map(|(key, value)| (key, value.into())).collect());
            Ok(())
        });
    }
}

impl LuaUserData for CreateRoomRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        general_macros!(methods);
        get!(methods, "get_name", name);
        get!(methods, "get_description", description);
        get_wrapper!(methods, "get_access_type", access_type, CreateRoomAccessTypeWrapper);
        get!(methods, "get_join_request", join_request);
        get!(methods, "get_max_user", max_user);
        get!(methods, "get_tags", tags);
        methods.add_method("get_metas", |_, this, ()| Ok(this.metas.clone().map(|meta| meta.into_iter().map(|(key, value)| (key, MetaTypeWrapper(value))).collect::<HashMap<String, MetaTypeWrapper<RoomMetaAccess>>>())));
        
        nullable_set!(methods, "set_name", name, String);
        nullable_set!(methods, "set_description", description, String);
        set_wrapper!(methods, "set_access_type", access_type, CreateRoomAccessTypeWrapper);
        set!(methods, "set_join_request", join_request, bool);
        set!(methods, "set_max_user", max_user, usize);

        methods.add_method_mut("set_tags", |_, this, tags: Vec<String>| {
            this.tags = tags;
            Ok(())
        });
        
        methods.add_method_mut("set_metas", |_, this, metas: Option<HashMap<String, MetaTypeWrapper<RoomMetaAccessWrapper>>>| {
            this.metas = metas.map(|meta| meta.into_iter().map(|(key, value)| (key, value.into())).collect());
            Ok(())
        });
    }
}

impl LuaUserData for UpdateRoom {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        general_macros!(methods);
        get!(methods, "get_name", name);
        get!(methods, "get_description", description);
        nullable_get!(methods, "get_access_type", access_type, CreateRoomAccessTypeWrapper);
        get!(methods, "get_join_request", join_request);
        get!(methods, "get_max_user", max_user);
        get!(methods, "get_tags", tags);
        methods.add_method("get_metas", |_, this, ()| Ok(this.metas.clone().map(|meta| meta.into_iter().map(|(key, value)| (key, MetaTypeWrapper(value))).collect::<HashMap<String, MetaTypeWrapper<RoomMetaAccess>>>())));
        methods.add_method("get_user_permission", |_, this, ()| Ok(this.user_permission.clone().map(|user_permission| user_permission.into_iter().map(|(key, value)| (UserIdWrapper(key), RoomUserTypeWrapper(value))).collect::<HashMap<UserIdWrapper, RoomUserTypeWrapper>>())));
        
        nullable_set!(methods, "set_name", name, String);
        nullable_set!(methods, "set_description", description, String);
        nullable_set!(methods, CreateRoomAccessTypeWrapper, "set_access_type", access_type, CreateRoomAccessType);
        nullable_set!(methods, bool, "set_join_request", join_request, bool);
        nullable_set!(methods, usize, "set_max_user", max_user, usize);
        nullable_set!(methods, Vec::<String>, "set_tags", tags, Vec::<String>);

        methods.add_method_mut("set_user_permission", |_, this, user_permission: Option<HashMap<UserIdWrapper, RoomUserTypeWrapper>>| {
            this.user_permission = user_permission.map(|permission| permission.into_iter().map(|(key, value)| (key.0, value.0)).collect());
            Ok(())
        });

        methods.add_method_mut("set_tags", |_, this, tags: Option<Vec<String>>| {
            this.tags = tags;
            Ok(())
        });
        
        methods.add_method_mut("set_metas", |_, this, metas: Option<HashMap<String, MetaTypeWrapper<RoomMetaAccessWrapper>>>| {
            this.metas = metas.map(|meta| meta.into_iter().map(|(key, value)| (key, value.into())).collect());
            Ok(())
        });
    }
}

impl LuaUserData for JoinToRoomRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        general_macros!(methods);
        get_wrapper!(methods, "get_room", room, RoomIdWrapper);
        get_wrapper!(methods, "get_room_user_type", room_user_type, RoomUserTypeWrapper);

        set_wrapper!(methods, "set_room", room, RoomIdWrapper);
        set_wrapper!(methods, "set_room_user_type", room_user_type, RoomUserTypeWrapper);
    }
}

impl LuaUserData for ProcessWaitingUser {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        general_macros!(methods);
        get_wrapper!(methods, "get_room", room, RoomIdWrapper);
        get_wrapper!(methods, "get_user", user, UserIdWrapper);
        get!(methods, "get_status", status);

        set_wrapper!(methods, "set_room", room, RoomIdWrapper);
        set_wrapper!(methods, "set_user", user, UserIdWrapper);
        set!(methods, "set_status", status, bool);
    }
}

impl LuaUserData for KickUserFromRoom {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        general_macros!(methods);
        get_wrapper!(methods, "get_room", room, RoomIdWrapper);
        get_wrapper!(methods, "get_user", user, UserIdWrapper);
        get!(methods, "get_ban", ban);

        set_wrapper!(methods, "set_room", room, RoomIdWrapper);
        set_wrapper!(methods, "set_user", user, UserIdWrapper);
        set!(methods, "set_ban", ban, bool);
    }
}

impl LuaUserData for DisconnectFromRoomRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        general_macros!(methods);
        get_wrapper!(methods, "get_room", room, RoomIdWrapper);
        set_wrapper!(methods, "set_room", room, RoomIdWrapper);
    }
}

impl LuaUserData for MessageToRoomRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        general_macros!(methods);
        get_wrapper!(methods, "get_room", room, RoomIdWrapper);
        get!(methods, "get_message", message);
        set_wrapper!(methods, "set_room", room, RoomIdWrapper);
        set!(methods, "set_message", message, String);
    }
}

impl LuaUserData for RoomListRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        get!(methods, "get_tag", tag);
        methods.add_method("get_members", |_, this, ()| Ok(this.members.clone().into_iter().map(RoomInfoTypeVariantWrapper).collect::<Vec<RoomInfoTypeVariantWrapper>>()));

        set!(methods, "set_tag", tag, Option<String>);
        methods.add_method_mut("set_members", |_, this, members: Vec<RoomInfoTypeVariantWrapper>| {
            this.members = members.into_iter().map(|member| member.0).collect::<Vec<RoomInfoTypeVariant>>();
            Ok(())
        });
    }
}

impl LuaUserData for WaitingRoomJoins {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        general_macros!(methods);
        get_wrapper!(methods, "get_room", room, RoomIdWrapper);
        set_wrapper!(methods, "set_room", room, RoomIdWrapper);
    }
}

impl LuaUserData for GetRoomRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        general_macros!(methods);
        get_wrapper!(methods, "get_room", room, RoomIdWrapper);
        methods.add_method("get_members", |_, this, ()| Ok(this.members.clone().into_iter().map(RoomInfoTypeVariantWrapper).collect::<Vec<RoomInfoTypeVariantWrapper>>()));
        
        set_wrapper!(methods, "set_room", room, RoomIdWrapper);
        methods.add_method_mut("set_members", |_, this, members: Vec<RoomInfoTypeVariantWrapper>| {
            this.members = members.into_iter().map(|member| member.0).collect::<Vec<RoomInfoTypeVariant>>();
            Ok(())
        });
    }
}

/* **************************************************************************************************************** */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */
