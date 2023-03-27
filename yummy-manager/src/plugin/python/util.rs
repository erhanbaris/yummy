use std::fmt::Debug;

/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use yummy_model::meta::{UserMetaAccess, UserMetaType, MetaType, RoomMetaType, RoomMetaAccess};
use num_bigint::ToBigInt;
use num_traits::Zero;
use rustpython::vm::{VirtualMachine, PyObjectRef, PyRef, builtins::{PyBaseException, PyFloat, PyInt, PyStr, PyList}, PyResult};

use super::modules::model::_model::{UserMetaTypeWrapper, RoomMetaTypeWrapper};

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
pub struct MetaTypeUtil;

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* **************************************************************************************************************** */
impl MetaTypeUtil {
    pub fn parse_user_meta(vm: &VirtualMachine, obj: &PyObjectRef, access: UserMetaAccess) -> Result<UserMetaTypeWrapper, PyRef<PyBaseException>> {
            
        /* Parse float */
        if obj.class().fast_issubclass(vm.ctx.types.float_type) {
            return Ok(UserMetaTypeWrapper::new(UserMetaType::Number(obj.payload::<PyFloat>().unwrap().to_f64(), access)));
        }

        /* Parse bool */
        if obj.class().fast_issubclass(vm.ctx.types.bool_type) {
            return Ok(UserMetaTypeWrapper::new(UserMetaType::Bool(!obj.payload::<PyInt>().unwrap().as_bigint().is_zero(), access)));
        }

        /* Parse string */
        if obj.class().fast_issubclass(vm.ctx.types.str_type) {
            return Ok(UserMetaTypeWrapper::new(UserMetaType::String(obj.payload::<PyStr>().unwrap().as_str().to_string(), access)));
        }

        /* Parse integer */
        if obj.class().fast_issubclass(vm.ctx.types.int_type) {
            return Ok(UserMetaTypeWrapper::new(UserMetaType::Number(obj.payload::<PyInt>().unwrap().try_to_primitive::<i32>(vm).unwrap() as f64, access)));
        }

        /* Parse list items */
        if obj.class().fast_issubclass(vm.ctx.types.list_type) {
            let mut meta_list = Vec::new();
            let python_list = obj.payload::<PyList>().unwrap();
            for item in python_list.borrow_vec().iter() {
                meta_list.push(Self::parse_user_meta(vm, item, UserMetaAccess::User)?.data);
            }

            return Ok(UserMetaTypeWrapper::new(UserMetaType::List(Box::new(meta_list), access)));
        }

        /* Item is not valid or Null */
        Ok(UserMetaTypeWrapper { data: UserMetaType::Null })
    }

    pub fn parse_room_meta(vm: &VirtualMachine, obj: &PyObjectRef, access: RoomMetaAccess) -> Result<RoomMetaTypeWrapper, PyRef<PyBaseException>> {
            
        /* Parse float */
        if obj.class().fast_issubclass(vm.ctx.types.float_type) {
            return Ok(RoomMetaTypeWrapper::new(RoomMetaType::Number(obj.payload::<PyFloat>().unwrap().to_f64(), access)));
        }

        /* Parse bool */
        if obj.class().fast_issubclass(vm.ctx.types.bool_type) {
            return Ok(RoomMetaTypeWrapper::new(RoomMetaType::Bool(!obj.payload::<PyInt>().unwrap().as_bigint().is_zero(), access)));
        }

        /* Parse string */
        if obj.class().fast_issubclass(vm.ctx.types.str_type) {
            return Ok(RoomMetaTypeWrapper::new(RoomMetaType::String(obj.payload::<PyStr>().unwrap().as_str().to_string(), access)));
        }

        /* Parse integer */
        if obj.class().fast_issubclass(vm.ctx.types.int_type) {
            return Ok(RoomMetaTypeWrapper::new(RoomMetaType::Number(obj.payload::<PyInt>().unwrap().try_to_primitive::<i32>(vm).unwrap() as f64, access)));
        }

        /* Parse list items */
        if obj.class().fast_issubclass(vm.ctx.types.list_type) {
            let mut meta_list = Vec::new();
            let python_list = obj.payload::<PyList>().unwrap();
            for item in python_list.borrow_vec().iter() {
                meta_list.push(Self::parse_room_meta(vm, item, RoomMetaAccess::User)?.data);
            }

            return Ok(RoomMetaTypeWrapper::new(RoomMetaType::List(Box::new(meta_list), access)));
        }

        /* Item is not valid or Null */
        Ok(RoomMetaTypeWrapper { data: RoomMetaType::Null })
    }

    pub fn as_python_value<T: Default + Debug + PartialEq + Clone + From<i32>>(meta: &MetaType<T>, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        Self::inner_as_python_value(meta, vm)
    }

    fn inner_as_python_value<T: Default + Debug + PartialEq + Clone + From<i32>>(meta: &MetaType<T>, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        match meta {
            MetaType::Null => Ok(vm.ctx.none()),
            MetaType::Number(value, _) => Ok(vm.ctx.new_float(*value).into()),
            MetaType::String(value, _) => Ok(vm.ctx.new_str(&value[..]).into()),
            MetaType::Bool(value, _) => Ok(vm.ctx.new_bool(*value).into()),
            MetaType::List(value, _) => {
                let mut items = Vec::new();
                for item in value.iter() {
                    items.push(Self::inner_as_python_value(item, vm)?);
                }
                Ok(vm.ctx.new_list(items).into())
            },
        }
    }

    pub fn get_meta_type<T: Default + Debug + PartialEq + Clone + From<i32>>(meta: &MetaType<T>, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        match meta {
            MetaType::Null => Ok(vm.ctx.new_bigint(&0.to_bigint().unwrap()).into()),
            MetaType::Number(_, _) => Ok(vm.ctx.new_bigint(&1.to_bigint().unwrap()).into()),
            MetaType::String(_, _) => Ok(vm.ctx.new_bigint(&2.to_bigint().unwrap()).into()),
            MetaType::Bool(_, _) => Ok(vm.ctx.new_bigint(&3.to_bigint().unwrap()).into()),
            MetaType::List(_, _) => Ok(vm.ctx.new_bigint(&4.to_bigint().unwrap()).into()),
        }
    }
}

/* **************************************************************************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */