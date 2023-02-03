use std::fmt::Debug;

use mlua::*;
use mlua::prelude::*;

use crate::meta::*;
use crate::model::UserType;

impl<T: Default + Debug + PartialEq + Clone + From<i32>> MetaType<T> {
    pub fn as_lua_value<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let value = match self {
            MetaType::Null => LuaValue::Nil,
            MetaType::Number(value, _) => LuaValue::Number(*value),
            MetaType::String(value, _) => LuaValue::String(lua.create_string(&value)?),
            MetaType::Bool(value, _) => LuaValue::Boolean(value.clone()),
            MetaType::List(value, _) => {
                let table = lua.create_table()?;
                for item in value.iter() {
                    table.push(item.clone().as_lua_value(lua)?)?;
                }
                LuaValue::Table(table)
            },
        };
        Ok(value)
    }
}

impl<T> LuaUserData for MetaType<T> where T: Default + Debug + PartialEq + Clone + From<i32>, i32: std::convert::From<T> {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_value", |lua, this, ()| Ok(this.as_lua_value(lua)?));
        methods.add_method("get_access_level", |_, this, ()| Ok(i32::from(this.get_access_level())));
    }
}

impl<'lua> FromLua<'lua> for UserMetaAccess {
    fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        match lua_value {
            Value::Integer(value) => Ok(UserMetaAccess::from(value as i32)),
            _ => Err(mlua::Error::RuntimeError("Meta does not have support for 'Error' type.".to_string()))
        }
    }
}

impl<'lua> ToLua<'lua> for UserMetaAccess {
    fn to_lua(self, _: &'lua Lua) -> LuaResult<Value<'lua>> {
        Ok(Value::Integer(i32::from(self) as i64))
    }
}

impl<'lua> FromLua<'lua> for MetaAction {
    fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        match lua_value {
            Value::Integer(value) => Ok(MetaAction::try_from(value as i32).unwrap_or_default()),
            _ => Err(mlua::Error::RuntimeError("Meta action does not have support for 'Error' type.".to_string()))
        }
    }
}

impl<'lua> ToLua<'lua> for MetaAction {
    fn to_lua(self, _: &'lua Lua) -> LuaResult<Value<'lua>> {
        Ok(Value::Integer(self as i64))
    }
}

impl<'lua> FromLua<'lua> for UserType {
    fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        match lua_value {
            Value::Integer(value) => Ok(UserType::from(value as i32)),
            _ => Err(mlua::Error::RuntimeError("User type does not have support for 'Error' type.".to_string()))
        }
    }
}

impl<'lua> ToLua<'lua> for UserType {
    fn to_lua(self, _: &'lua Lua) -> LuaResult<Value<'lua>> {
        Ok(Value::Integer(self as i64))
    }
}
