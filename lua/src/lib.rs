use std::fmt::Debug;

use mlua::*;
use mlua::prelude::*;
use model::meta::*;
use model::*;
use cache::state::RoomInfoTypeVariant;

#[derive(Default, Debug, PartialEq, Eq, Clone, Hash)]
pub struct UserIdWrapper(pub UserId);
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct RoomIdWrapper(pub RoomId);
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct RoomUserTypeWrapper(pub RoomUserType);
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct MetaActionWrapper(pub MetaAction);
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct UserTypeWrapper(pub UserType);
pub struct MetaTypeWrapper<T>(pub MetaType<T>) where T: Default + Debug + PartialEq + Clone + From<i32>;
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct CreateRoomAccessTypeWrapper(pub CreateRoomAccessType);
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct UserMetaAccessWrapper(pub UserMetaAccess);
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct RoomMetaAccessWrapper(pub RoomMetaAccess);
pub struct RoomInfoTypeVariantWrapper(pub RoomInfoTypeVariant);

impl From<i32> for UserMetaAccessWrapper {
    fn from(access: i32) -> Self {
        UserMetaAccessWrapper(UserMetaAccess::from(access))
    }
}

impl From<UserMetaAccessWrapper> for i32 {
    fn from(meta: UserMetaAccessWrapper) -> Self {
        meta.0.into()
    }
}

impl From<i32> for RoomMetaAccessWrapper {
    fn from(access: i32) -> Self {
        RoomMetaAccessWrapper(RoomMetaAccess::from(access))
    }
}

impl From<RoomMetaAccessWrapper> for i32 {
    fn from(meta: RoomMetaAccessWrapper) -> Self {
        meta.0.into()
    }
}

/*impl<T> From<MetaTypeWrapper<T>> for MetaType<T> where T: Default + Debug + PartialEq + Clone + From<i32> {
    fn from(item: MetaTypeWrapper<T>) -> Self {
        match item.0 {
            MetaType::Null => MetaType::Null,
            MetaType::Number(value, access) => MetaType::Number(value, access),
            MetaType::String(value, access) => MetaType::String(value, access),
            MetaType::Bool(value, access) => MetaType::Bool(value, access),
            MetaType::List(value, access) => {
                let mut items: Vec<MetaType<T>> = Vec::new();
                for item in value.into_iter() {
                    items.push(item);
                }
                MetaType::List(Box::new(items), access)
            },
        }
    }
}*/

fn convert_user_meta(item: MetaType<UserMetaAccessWrapper>) -> MetaType<UserMetaAccess> {
    match item {
        MetaType::Null => MetaType::Null,
        MetaType::Number(value, access) => MetaType::Number(value, access.0),
        MetaType::String(value, access) => MetaType::String(value, access.0),
        MetaType::Bool(value, access) => MetaType::Bool(value, access.0),
        MetaType::List(value, access) => {
            let mut items: Vec<MetaType<UserMetaAccess>> = Vec::new();
            for item in value.into_iter() {
                items.push(convert_user_meta(item));
            }
            MetaType::List(Box::new(items), access.0)
        },
    }
}

impl From<MetaTypeWrapper<UserMetaAccessWrapper>> for MetaType<UserMetaAccess> {
    fn from(item: MetaTypeWrapper<UserMetaAccessWrapper>) -> Self {
        match item.0 {
            MetaType::Null => MetaType::Null,
            MetaType::Number(value, access) => MetaType::Number(value, access.0),
            MetaType::String(value, access) => MetaType::String(value, access.0),
            MetaType::Bool(value, access) => MetaType::Bool(value, access.0),
            MetaType::List(value, access) => {
                let mut items: Vec<MetaType<UserMetaAccess>> = Vec::new();
                for item in value.into_iter() {
                    items.push(convert_user_meta(item));
                }
                MetaType::List(Box::new(items), access.0)
            },
        }
    }
}

fn convert_room_meta(item: MetaType<RoomMetaAccessWrapper>) -> MetaType<RoomMetaAccess> {
    match item {
        MetaType::Null => MetaType::Null,
        MetaType::Number(value, access) => MetaType::Number(value, access.0),
        MetaType::String(value, access) => MetaType::String(value, access.0),
        MetaType::Bool(value, access) => MetaType::Bool(value, access.0),
        MetaType::List(value, access) => {
            let mut items: Vec<MetaType<RoomMetaAccess>> = Vec::new();
            for item in value.into_iter() {
                items.push(convert_room_meta(item));
            }
            MetaType::List(Box::new(items), access.0)
        },
    }
}

impl From<MetaTypeWrapper<RoomMetaAccessWrapper>> for MetaType<RoomMetaAccess> {
    fn from(item: MetaTypeWrapper<RoomMetaAccessWrapper>) -> Self {
        match item.0 {
            MetaType::Null => MetaType::Null,
            MetaType::Number(value, access) => MetaType::Number(value, access.0),
            MetaType::String(value, access) => MetaType::String(value, access.0),
            MetaType::Bool(value, access) => MetaType::Bool(value, access.0),
            MetaType::List(value, access) => {
                let mut items: Vec<MetaType<RoomMetaAccess>> = Vec::new();
                for item in value.into_iter() {
                    items.push(convert_room_meta(item));
                }
                MetaType::List(Box::new(items), access.0)
            },
        }
    }
}

impl From<CreateRoomAccessTypeWrapper> for CreateRoomAccessType {
    fn from(access: CreateRoomAccessTypeWrapper) -> Self {
        access.0
    }
}

impl<T: Default + Debug + PartialEq + Clone + From<i32>> MetaTypeWrapper<T> where i32: std::convert::From<T> {
    pub fn as_lua_value<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        MetaTypeWrapper::inner_as_lua_value(&self.0, lua)
    }

    fn inner_as_lua_value<'lua>(item: &MetaType<T>, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let value = match item {
            MetaType::Null => LuaValue::Nil,
            MetaType::Number(value, _) => LuaValue::Number(*value),
            MetaType::String(value, _) => LuaValue::String(lua.create_string(&value)?),
            MetaType::Bool(value, _) => LuaValue::Boolean(*value),
            MetaType::List(value, _) => {
                let table = lua.create_table()?;
                for (index, item) in value.iter().enumerate() {
                    table.set(index + 1, MetaTypeWrapper::inner_as_lua_value(item, lua)?)?;
                }
                LuaValue::Table(table)
            },
        };
        Ok(value)
    }

    pub fn get_type<'lua>(&self) -> LuaResult<LuaValue<'lua>> {
        let value = match self.0 {
            MetaType::Null => LuaValue::Integer(0),
            MetaType::Number(_, _) => LuaValue::Integer(1),
            MetaType::String(_, _) => LuaValue::Integer(2),
            MetaType::Bool(_, _) => LuaValue::Integer(3),
            MetaType::List(_, _) => LuaValue::Integer(4),
        };
        Ok(value)
    }
}

impl<T> LuaUserData for MetaTypeWrapper<T> where T: Default + Debug + PartialEq + Clone + From<i32>, i32: std::convert::From<T> {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_type", |_, this, ()| this.get_type());
        methods.add_method("get_value", |lua, this, ()| this.as_lua_value(lua));
        methods.add_method("get_access_level", |_, this, ()| Ok(i32::from(this.0.get_access_level())));
    }
}

impl<'lua> FromLua<'lua> for UserMetaAccessWrapper {
    fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        match lua_value {
            Value::Integer(value) => Ok(UserMetaAccessWrapper(UserMetaAccess::from(value as i32))),
            _ => Err(mlua::Error::RuntimeError("Meta does not have support for 'Error' type.".to_string()))
        }
    }
}

impl<'lua, T> FromLua<'lua> for MetaTypeWrapper<T> where T: Default + Debug + PartialEq + Clone + From<i32> + 'static, i32: std::convert::From<T> {
    fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        match lua_value {
            Value::UserData(value) => Ok(MetaTypeWrapper(value.borrow::<Self>()?.0.clone())),
            _ => Err(mlua::Error::RuntimeError("Meta does not have support for 'Error' type.".to_string()))
        }
    }
}

impl<'lua> FromLua<'lua> for RoomMetaAccessWrapper {
    fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        match lua_value {
            Value::Integer(value) => Ok(RoomMetaAccessWrapper(RoomMetaAccess::from(value as i32))),
            _ => Err(mlua::Error::RuntimeError("Meta does not have support for 'Error' type.".to_string()))
        }
    }
}

impl<'lua> ToLua<'lua> for RoomMetaAccessWrapper {
    fn to_lua(self, _: &'lua Lua) -> LuaResult<Value<'lua>> {
        Ok(Value::Integer(i32::from(self.0) as i64))
    }
}

impl<'lua> ToLua<'lua> for MetaActionWrapper {
    fn to_lua(self, _: &'lua Lua) -> LuaResult<Value<'lua>> {
        Ok(Value::Integer(self.0 as i64))
    }
}

impl<'lua> ToLua<'lua> for UserTypeWrapper {
    fn to_lua(self, _: &'lua Lua) -> LuaResult<Value<'lua>> {
        Ok(Value::Integer(self.0 as i64))
    }
}

impl<'lua> FromLua<'lua> for CreateRoomAccessTypeWrapper {
    fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        match lua_value {
            Value::Integer(value) => Ok(CreateRoomAccessTypeWrapper(CreateRoomAccessType::from(value as i32))),
            _ => Err(mlua::Error::RuntimeError("Crate room access does not have support for 'Error' type.".to_string()))
        }
    }
}

impl<'lua> ToLua<'lua> for CreateRoomAccessTypeWrapper {
    fn to_lua(self, _: &'lua Lua) -> LuaResult<Value<'lua>> {
        Ok(Value::Integer(self.0 as i64))
    }
}

impl<'lua> FromLua<'lua> for RoomUserTypeWrapper {
    fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        match lua_value {
            Value::Integer(value) => Ok(RoomUserTypeWrapper(RoomUserType::from(value as i32))),
            _ => Err(mlua::Error::RuntimeError("Room user type does not have support for 'Error' type.".to_string()))
        }
    }
}

impl<'lua> ToLua<'lua> for RoomUserTypeWrapper {
    fn to_lua(self, _: &'lua Lua) -> LuaResult<Value<'lua>> {
        Ok(Value::Integer(self.0 as i64))
    }
}

impl<'lua> FromLua<'lua> for UserIdWrapper {
    fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        match lua_value {
            Value::String(value) => {
                match value.to_str() {
                    Ok(user_id) => Ok(UserIdWrapper(UserId::from(user_id.to_string()))),
                    Err(error) => Err(mlua::Error::RuntimeError(error.to_string()))
                }
            },
            _ => Err(mlua::Error::RuntimeError("Room user type does not have support for 'Error' type.".to_string()))
        }
    }
}

impl<'lua> ToLua<'lua> for UserIdWrapper {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<Value<'lua>> {
        Ok(Value::String(lua.create_string(&self.0.to_string())?))
    }
}

impl<'lua> FromLua<'lua> for RoomIdWrapper {
    fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        match lua_value {
            Value::String(value) => {
                match value.to_str() {
                    Ok(room_id) => Ok(RoomIdWrapper(RoomId::from(room_id.to_string()))),
                    Err(error) => Err(mlua::Error::RuntimeError(error.to_string()))
                }
            },
            _ => Err(mlua::Error::RuntimeError("Room does not have support for 'Error' type.".to_string()))
        }
    }
}

impl<'lua> ToLua<'lua> for RoomIdWrapper {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<Value<'lua>> {
        Ok(Value::String(lua.create_string(&self.0.to_string())?))
    }
}

impl<'lua> FromLua<'lua> for RoomInfoTypeVariantWrapper {
    fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        match lua_value {
            Value::Integer(value) => Ok(RoomInfoTypeVariantWrapper(match value {
                0 => RoomInfoTypeVariant::RoomName,
                1 => RoomInfoTypeVariant::Description,
                2 => RoomInfoTypeVariant::Users,
                3 => RoomInfoTypeVariant::MaxUser,
                4 => RoomInfoTypeVariant::UserLength,
                5 => RoomInfoTypeVariant::AccessType,
                6 => RoomInfoTypeVariant::Tags,
                7 => RoomInfoTypeVariant::Metas,
                8 => RoomInfoTypeVariant::InsertDate,
                9 => RoomInfoTypeVariant::JoinRequest,
                10 => RoomInfoTypeVariant::BannedUsers,
                _ => return Err(mlua::Error::RuntimeError("Room info type not valid.".to_string()))
            })),
            _ => Err(mlua::Error::RuntimeError("Room does not have support for 'Error' type.".to_string()))
        }
    }
}

impl<'lua> ToLua<'lua> for RoomInfoTypeVariantWrapper {
    fn to_lua(self, _: &'lua Lua) -> LuaResult<Value<'lua>> {
        Ok(Value::Integer(self.0 as i64))
    }
}
