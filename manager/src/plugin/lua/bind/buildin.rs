/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::fmt::Debug;

use lua::MetaTypeWrapper;
use lua::RoomMetaAccessWrapper;
use lua::UserMetaAccessWrapper;
use mlua::Lua;
use mlua::prelude::*;

use model::meta::MetaType;

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* **************************************************************************************************************** */
pub fn inner_bind_buildin_functions(lua: &mut Lua) -> LuaResult<()> {
    let globals = lua.globals();
    globals.set("new_user_meta", lua.create_function(new_user_meta)?)?;
    globals.set("new_room_meta", lua.create_function(new_room_meta)?)?;
    Ok(())
}

fn lua_to_metawrapper<T: Default + Debug + PartialEq + Clone + From<i32>>(_lua: &'_ Lua, (value, access): (LuaValue, T)) -> LuaResult<MetaTypeWrapper<T>> {
    match value {
        LuaValue::Nil => Ok(MetaTypeWrapper(MetaType::Null)),
        LuaValue::Boolean(val) => Ok(MetaTypeWrapper(MetaType::Bool(val, access))),
        LuaValue::LightUserData(_) => Ok(MetaTypeWrapper(MetaType::Null)),
        LuaValue::Integer(val) => Ok(MetaTypeWrapper(MetaType::Number(val as f64, access))),
        LuaValue::Number(val) => Ok(MetaTypeWrapper(MetaType::Number(val, access))),
        LuaValue::String(val) =>  Ok(MetaTypeWrapper(MetaType::String(val.to_str().unwrap_or_default().to_string(), access))),
        LuaValue::Table(table) => {
            let mut array = Vec::new();
            
            for row in table.sequence_values::<LuaValue>() {
                let row = row?;
                let row = lua_to_meta(_lua, (row, T::default()))?;
                array.push(row);
            }

            Ok(MetaTypeWrapper(MetaType::List(Box::new(array), access)))
        },
        LuaValue::Function(_) => Err(mlua::Error::RuntimeError("Meta does not have support for 'Function' type.".to_string())),
        LuaValue::Thread(_) => Err(mlua::Error::RuntimeError("Meta does not have support for 'Thread' type.".to_string())),
        LuaValue::UserData(_) => Err(mlua::Error::RuntimeError("Meta does not have support for 'UserData' type.".to_string())),
        LuaValue::Error(_) => Err(mlua::Error::RuntimeError("Meta does not have support for 'Error' type.".to_string())),
    }
}

fn lua_to_meta<T: Default + Debug + PartialEq + Clone + From<i32>>(_lua: &'_ Lua, (value, access): (LuaValue, T)) -> LuaResult<MetaType<T>> {
    match value {
        LuaValue::Nil => Ok(MetaType::Null),
        LuaValue::Boolean(val) => Ok(MetaType::Bool(val, access)),
        LuaValue::LightUserData(_) => Ok(MetaType::Null),
        LuaValue::Integer(val) => Ok(MetaType::Number(val as f64, access)),
        LuaValue::Number(val) => Ok(MetaType::Number(val, access)),
        LuaValue::String(val) =>  Ok(MetaType::String(val.to_str().unwrap_or_default().to_string(), access)),
        LuaValue::Table(table) => {
            let mut array = Vec::new();
            
            for row in table.sequence_values::<LuaValue>() {
                let row = row?;
                let row = lua_to_meta(_lua, (row, T::default()))?;
                array.push(row);
            }

            Ok(MetaType::List(Box::new(array), access))
        },
        LuaValue::Function(_) => Err(mlua::Error::RuntimeError("Meta does not have support for 'Function' type.".to_string())),
        LuaValue::Thread(_) => Err(mlua::Error::RuntimeError("Meta does not have support for 'Thread' type.".to_string())),
        LuaValue::UserData(_) => Err(mlua::Error::RuntimeError("Meta does not have support for 'UserData' type.".to_string())),
        LuaValue::Error(_) => Err(mlua::Error::RuntimeError("Meta does not have support for 'Error' type.".to_string())),
    }
}

fn new_user_meta<'a>(lua: &'a Lua, (value, access): (LuaValue, UserMetaAccessWrapper)) -> LuaResult<LuaValue<'a>> {
    let value = lua.create_userdata(lua_to_metawrapper(lua, (value, access))?)?;
    Ok(LuaValue::UserData(value))
}

fn new_room_meta<'a>(lua: &'a Lua, (value, access): (LuaValue, RoomMetaAccessWrapper)) -> LuaResult<LuaValue<'a>> {
    let value = lua.create_userdata(lua_to_metawrapper(lua, (value, access))?)?;
    Ok(LuaValue::UserData(value))
}

/* **************************************************************************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */
