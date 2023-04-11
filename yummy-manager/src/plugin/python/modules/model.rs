use rustpython_vm::{VirtualMachine, PyObjectRef, extend_module};
use rustpython_vm::class::PyClassImpl;
use crate::plugin::python::YummyPluginContextWrapper;

use self::model::*;

use super::base::_base::PyYummyValidationError;

pub fn make_module(vm: &VirtualMachine) -> PyObjectRef {
    let module = model::make_module(vm);

    PyYummyValidationError::extend_class(&vm.ctx, vm.ctx.exceptions.base_exception_type);

    extend_module!(vm, module, {
        "DeviceIdAuth" => DeviceIdAuthRequestWrapper::make_class(&vm.ctx),
        "EmailAuth" => EmailAuthRequestWrapper::make_class(&vm.ctx),
        "CustomIdAuth" => CustomIdAuthRequestWrapper::make_class(&vm.ctx),
        "Logout" => LogoutRequestWrapper::make_class(&vm.ctx),
        "UserConnected" => UserConnectedWrapper::make_class(&vm.ctx),
        "UserDisconnected" => ConnUserDisconnectWrapper::make_class(&vm.ctx),
        "RefreshToken" => RefreshTokenRequestWrapper::make_class(&vm.ctx),
        "RestoreToken" => RestoreTokenRequestWrapper::make_class(&vm.ctx),
        "GetUserInformation" => GetUserInformationWrapper::make_class(&vm.ctx),
        "UpdateUser" => UpdateUserWrapper::make_class(&vm.ctx),
        "CreateRoom" => CreateRoomRequestWrapper::make_class(&vm.ctx),
        "UpdateRoom" => UpdateRoomWrapper::make_class(&vm.ctx),
        "JoinToRoom" => JoinToRoomRequestWrapper::make_class(&vm.ctx),
        "ProcessWaitingUser" => ProcessWaitingUserWrapper::make_class(&vm.ctx),
        "KickUserFromRoom" => KickUserFromRoomWrapper::make_class(&vm.ctx),
        "DisconnectFromRoom" => DisconnectFromRoomRequestWrapper::make_class(&vm.ctx),
        "MessageToRoom" => MessageToRoomRequestWrapper::make_class(&vm.ctx),
        "RoomListRequest" => RoomListRequestWrapper::make_class(&vm.ctx),
        "WaitingRoomJoins" => WaitingRoomJoinsWrapper::make_class(&vm.ctx),
        "GetRoomRequest" => GetRoomRequestWrapper::make_class(&vm.ctx),
        "YummyPluginContext" => YummyPluginContextWrapper::make_class(&vm.ctx),
    });

    module
}

#[rustpython::vm::pymodule(name = "model")]
pub mod model {

    /* **************************************************************************************************************** */
    /* **************************************************** MODS ****************************************************** */
    /* *************************************************** IMPORTS **************************************************** */
    /* **************************************************************************************************************** */
    use std::{rc::Rc, cell::RefCell};
    use std::ops::Deref;
    use std::collections::HashMap;
    
    use num_bigint::ToBigInt;

    use rustpython_vm::builtins::{PyBaseException, PyInt, PyDict, PyStr};
    use rustpython_vm::{VirtualMachine, PyResult, PyObjectRef, TryFromBorrowedObject, PyRef, PyObject, py_serde};
    use yummy_cache::state::RoomInfoTypeVariant;
    use yummy_general::password::Password;
    use yummy_model::{UserType, CreateRoomAccessType, UserId, RoomUserType};
    use yummy_model::meta::{MetaAction, RoomMetaType, RoomMetaAccess};
    use yummy_model::{meta::{UserMetaAccess, UserMetaType}};
    use yummy_macros::yummy_model;
    use rustpython::vm::{pyclass, PyPayload};
    use rustpython::vm::builtins::PyIntRef;
    use rustpython_vm::class::PyClassImpl;

    use crate::plugin::python::modules::base::_base::PyYummyValidationError;
    use crate::plugin::python::util::MetaTypeUtil;
    use crate::room::model::{UpdateRoom, JoinToRoomRequest, ProcessWaitingUser, KickUserFromRoom, DisconnectFromRoomRequest, MessageToRoomRequest, RoomListRequest, WaitingRoomJoins, GetRoomRequest};
    use crate::{auth::model::{DeviceIdAuthRequest, EmailAuthRequest, CustomIdAuthRequest, ConnUserDisconnect, LogoutRequest, RefreshTokenRequest, RestoreTokenRequest}, conn::model::UserConnected, user::model::{UpdateUser, GetUserInformation, GetUserInformationEnum}, room::model::CreateRoomRequest};
    use crate::plugin::python::ModelWrapper;

    /* **************************************************************************************************************** */
    /* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
    /* **************************************************** MACROS **************************************************** */
    /* **************************************************************************************************************** */

    macro_rules! model_wrapper_struct {
        ($model: ident, $wrapper: ident, $class_name: expr) => {
            #[pyclass(module = false, name = $class_name)]
            #[derive(Debug, PyPayload)]
            pub struct $wrapper {
                pub data: Rc<RefCell< $model >>
            }

            impl ModelWrapper for $wrapper {
                type Entity = $model;
                fn wrap(entity: Rc<RefCell<Self::Entity>>) -> Self {
                    $wrapper::new(entity)
                }
            }

            unsafe impl Send for $wrapper {}
            unsafe impl Sync for $wrapper {}
        }
    }

    macro_rules! wrapper_struct {
        ($model: ident, $wrapper: ident, $class_name: expr) => {
            #[pyclass(module = false, name = $class_name)]
            #[derive(Debug, PyPayload)]
            pub struct $wrapper {
                pub data: $model
            }

            impl $wrapper {
                pub fn new(data: $model) -> Self {
                    Self { data }
                }
            }

            unsafe impl Send for $wrapper {}
            unsafe impl Sync for $wrapper {}
        }
    }

    macro_rules! get_string {
        ($self: expr, $item: ident, $vm: ident) => {
            Ok($vm.ctx.new_str(&$self.data.borrow().$item[..]).into())
        };
    }

    macro_rules! get_bool {
        ($self: expr, $item: ident, $vm: ident) => {
            Ok($vm.ctx.new_bool($self.data.borrow().$item).into())
        };
    }

    macro_rules! get_usize {
        ($self: expr, $item: ident, $vm: ident) => {
            Ok($vm.ctx.new_bigint(&($self.data.borrow().$item as u32).to_bigint().unwrap()).into())
        };
    }

    macro_rules! set_value {
        ($self: expr,  $target: ident, $source: expr) => {
            $self.data.borrow_mut().$target = $source;
        };
    }

    macro_rules! set_nullable_usize {
        ($self: expr, $target: ident, $number: ident) => {
            $self.data.borrow_mut().$target = $number.map(|item| item.as_u32_mask() as usize);
        };
    }

    macro_rules! get_nullable_f64 {
        ($self: expr, $target: ident, $vm: ident) => {
            match $self.data.borrow().$target {
                Some(data) => Ok($vm.ctx.new_float(data as f64).into()),
                None => Ok($vm.ctx.none())
            }
        };
    }

    macro_rules! get_nullable_bool {
        ($self: expr, $target: ident, $vm: ident) => {
            match $self.data.borrow().$target {
                Some(data) => Ok($vm.ctx.new_bool(data).into()),
                None => Ok($vm.ctx.none())
            }
        };
    }

    macro_rules! get_nullable_usize {
        ($self: expr, $target: ident, $vm: ident) => {
            match $self.data.borrow().$target {
                Some(data) => Ok($vm.ctx.new_bigint(&data.to_bigint().unwrap()).into()),
                None => Ok($vm.ctx.none())
            }
        };
    }

    macro_rules! get_nullable_string {
        ($self: expr, $target: ident, $vm: ident) => {
            match &$self.data.borrow().$target {
                Some(data) => Ok($vm.ctx.new_str(&data.to_string()[..]).into()),
                None => Ok($vm.ctx.none())
            }
        };
    }

    macro_rules! get_user_id {
        ($self: expr, $vm: ident) => {
            match $self.data.borrow().auth.deref() {
                Some(auth) => Ok($vm.ctx.new_str(&auth.user.to_string()[..]).into()),
                None => Ok($vm.ctx.none())
            }
        };
    }

    macro_rules! get_session_id {
        ($self: expr, $vm: ident) => {
            match $self.data.borrow().auth.deref() {
                Some(auth) => Ok($vm.ctx.new_str(&auth.session.to_string()[..]).into()),
                None => Ok($vm.ctx.none())
            }
        };
    }

    /* **************************************************************************************************************** */
    /* *************************************************** STRUCTS **************************************************** */
    /* **************************************************************************************************************** */
    model_wrapper_struct!(DeviceIdAuthRequest, DeviceIdAuthRequestWrapper, "DeviceIdAuth");
    model_wrapper_struct!(EmailAuthRequest, EmailAuthRequestWrapper, "EmailAuth");
    model_wrapper_struct!(CustomIdAuthRequest, CustomIdAuthRequestWrapper, "CustomIdAuth");
    model_wrapper_struct!(UserConnected, UserConnectedWrapper, "UserConnected");
    model_wrapper_struct!(ConnUserDisconnect, ConnUserDisconnectWrapper, "ConnUserDisconnect");
    model_wrapper_struct!(UpdateUser, UpdateUserWrapper, "UpdateUser");
    model_wrapper_struct!(LogoutRequest, LogoutRequestWrapper, "Logout");
    model_wrapper_struct!(RefreshTokenRequest, RefreshTokenRequestWrapper, "RefreshToken");
    model_wrapper_struct!(RestoreTokenRequest, RestoreTokenRequestWrapper, "RestoreToken");
    model_wrapper_struct!(GetUserInformation, GetUserInformationWrapper, "GetUserInformation");
    model_wrapper_struct!(CreateRoomRequest, CreateRoomRequestWrapper, "CreateRoom");
    model_wrapper_struct!(UpdateRoom, UpdateRoomWrapper, "UpdateRoom");
    model_wrapper_struct!(JoinToRoomRequest, JoinToRoomRequestWrapper, "JoinToRoom");
    model_wrapper_struct!(ProcessWaitingUser, ProcessWaitingUserWrapper, "ProcessWaitingUser");
    model_wrapper_struct!(KickUserFromRoom, KickUserFromRoomWrapper, "KickUserFromRoom");
    model_wrapper_struct!(DisconnectFromRoomRequest, DisconnectFromRoomRequestWrapper, "DisconnectFromRoom");
    model_wrapper_struct!(MessageToRoomRequest, MessageToRoomRequestWrapper, "MessageToRoom");
    model_wrapper_struct!(RoomListRequest, RoomListRequestWrapper, "RoomListRequest");
    model_wrapper_struct!(WaitingRoomJoins, WaitingRoomJoinsWrapper, "WaitingRoomJoins");
    model_wrapper_struct!(GetRoomRequest, GetRoomRequestWrapper, "GetRoomRequest");

    wrapper_struct!(UserMetaType, UserMetaTypeWrapper, "UserMetaType");
    wrapper_struct!(RoomMetaType, RoomMetaTypeWrapper, "RoomMetaType");
    wrapper_struct!(UserMetaAccess, UserMetaAccessWrapper, "UserMetaAccess");
    wrapper_struct!(RoomMetaAccess, RoomMetaAccessWrapper, "RoomMetaAccess");
    wrapper_struct!(RoomInfoTypeVariant, RoomInfoTypeVariantWrapper, "RoomInfoTypeVariant");
    
    wrapper_struct!(CreateRoomAccessType, CreateRoomAccessTypeWrapper, "CreateRoomAccessType");

    /* **************************************************************************************************************** */
    /* **************************************************** ENUMS ***************************************************** */
    /* ************************************************** FUNCTIONS *************************************************** */
    /* *************************************************** TRAITS ***************************************************** */
    /* ************************************************* IMPLEMENTS *************************************************** */
    /* **************************************************************************************************************** */
    #[yummy_model(class_name="UserConnected", no_request_id=true, no_auth=true)]
    #[pyclass(flags(BASETYPE))]
    impl UserConnectedWrapper {
        #[pymethod]
        pub fn get_user_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_str(&self.data.borrow().user_id.to_string()[..]).into())
        }
    }

    #[yummy_model(class_name="RefreshTokenRequest")]
    #[pyclass(flags(BASETYPE))]
    impl RefreshTokenRequestWrapper {
        #[pymethod]
        pub fn get_token(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_str(&self.data.borrow().token.to_string()[..]).into())
        }
    }

    #[yummy_model(class_name="RestoreTokenRequest")]
    #[pyclass(flags(BASETYPE))]
    impl RestoreTokenRequestWrapper {
        #[pymethod]
        pub fn get_token(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_str(&self.data.borrow().token.to_string()[..]).into())
        }
    }

    #[yummy_model(class_name="GetUserInformation", no_auth=true)]
    #[pyclass(flags(BASETYPE))]
    impl GetUserInformationWrapper {
        #[pymethod]
        pub fn get_query_type(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            match self.data.borrow().query {
                GetUserInformationEnum::Me(_) => Ok(vm.ctx.new_str("Me").into()),
                GetUserInformationEnum::UserViaSystem(_) => Ok(vm.ctx.new_str("UserViaSystem").into()),
                GetUserInformationEnum::User { user: _, requester: _ } => Ok(vm.ctx.new_str("User").into()),
            }
        }

        #[pymethod]
        pub fn get_user_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            match &self.data.borrow().query {
                GetUserInformationEnum::Me(user) => {
                    match user.as_ref() {
                        Some(auth) => Ok(vm.ctx.new_str(&auth.user.to_string()[..]).into()),
                        None => Ok(vm.ctx.none())
                    }
                },
                GetUserInformationEnum::UserViaSystem(user) => Ok(vm.ctx.new_str(&user.to_string()[..]).into()),
                GetUserInformationEnum::User { user, requester: _ } => Ok(vm.ctx.new_str(&user.to_string()[..]).into())
            }
        }

        #[pymethod]
        pub fn get_requester_user_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            match &self.data.borrow().query {
                GetUserInformationEnum::Me(_) => Ok(vm.ctx.none()),
                GetUserInformationEnum::UserViaSystem(_) => Ok(vm.ctx.none()),
                GetUserInformationEnum::User { user: _, requester } => {
                    match requester.as_ref() {
                        Some(auth) => Ok(vm.ctx.new_str(&auth.user.to_string()[..]).into()),
                        None => Ok(vm.ctx.none())
                    }
                }
            }
        }

        #[pymethod]
        pub fn get_value(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            match &self.data.borrow().query {
                GetUserInformationEnum::Me(user) => {
                    match user.as_ref() {
                        Some(auth) => Ok(vm.ctx.new_tuple(vec![vm.ctx.new_str(&auth.user.to_string()[..]).into(), vm.ctx.none()]).into()),
                        None => Ok(vm.ctx.new_tuple(vec![vm.ctx.none(), vm.ctx.none()]).into())
                    }
                },
                GetUserInformationEnum::UserViaSystem(user) => Ok(vm.ctx.new_tuple(vec![vm.ctx.new_str(&user.to_string()[..]).into(), vm.ctx.none()]).into()),
                GetUserInformationEnum::User { user, requester } => {
                    match requester.as_ref() {
                        Some(auth) => Ok(vm.ctx.new_tuple(vec![vm.ctx.new_str(&user.to_string()[..]).into(), vm.ctx.new_str(&auth.user.to_string()[..]).into()]).into()),
                        None => Ok(vm.ctx.new_tuple(vec![vm.ctx.new_str(&user.to_string()[..]).into(), vm.ctx.none()]).into())
                    }
                }
            }
        }
    }

    #[yummy_model(class_name="ConnUserDisconnect")]
    #[pyclass(flags(BASETYPE))]
    impl ConnUserDisconnectWrapper {
        #[pymethod]
        pub fn get_send_message(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_bool!(self, send_message, vm)
        }

        #[pymethod]
        pub fn set_send_message(&self, send_message: bool) -> PyResult<()> {
            set_value!(self, send_message, send_message);
            Ok(())
        }
    }

    #[yummy_model(class_name="LogoutRequest")]
    #[pyclass(flags(BASETYPE))]
    impl LogoutRequestWrapper {}

    #[yummy_model(class_name="UpdateUser")]
    #[pyclass(flags(BASETYPE))]
    impl UpdateUserWrapper {
        /* Name functions */
        #[pymethod]
        pub fn get_name(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_nullable_string!(self, name, vm)
        }

        #[pymethod]
        pub fn set_name(&self, name: Option<String>) -> PyResult<()> {
            set_value!(self, name, name);
            Ok(())
        }

        /* TargetUserId functions */
        #[pymethod]
        pub fn get_target_user_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_nullable_string!(self, target_user_id, vm)
        }

        /* Email functions */
        #[pymethod]
        pub fn get_email(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_nullable_string!(self, email, vm)
        }

        #[pymethod]
        pub fn set_email(&self, email: Option<String>) -> PyResult<()> {
            set_value!(self, email, email);
            Ok(())
        }

        /* Password functions */
        #[pymethod]
        pub fn get_password(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_nullable_string!(self, password, vm)
        }

        #[pymethod]
        pub fn set_password(&self, password: Option<String>) -> PyResult<()> {
            set_value!(self, password, password);
            Ok(())
        }

        /* DeviceID functions */
        #[pymethod]
        pub fn get_device_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_nullable_string!(self, device_id, vm)
        }

        #[pymethod]
        pub fn set_device_id(&self, device_id: Option<String>) -> PyResult<()> {
            set_value!(self, device_id, device_id);
            Ok(())
        }

        /* CustomID functions */
        #[pymethod]
        pub fn get_custom_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_nullable_string!(self, custom_id, vm)
        }

        #[pymethod]
        pub fn set_custom_id(&self, custom_id: Option<String>) -> PyResult<()> {
            set_value!(self, custom_id, custom_id);
            Ok(())
        }

        /* UserType functions */
        #[pymethod]
        pub fn get_user_type(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            match self.data.borrow().user_type {
                Some(data) => Ok(vm.ctx.new_bigint(&(data as u32).to_bigint().unwrap()).into()),
                None => Ok(vm.ctx.none())
            }
        }

        #[pymethod]
        pub fn set_user_type(&self, user_type: Option<i32>) -> PyResult<()> {
            self.data.borrow_mut().user_type = user_type.map(UserType::from);
            Ok(())
        }

        /* MetaAction functions */
        #[pymethod]
        pub fn get_meta_action(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_bigint(&(self.data.borrow().meta_action.clone() as u32).to_bigint().unwrap()).into())
        }

        #[pymethod]
        pub fn set_meta_action(&self, meta_action: i32) -> PyResult<()> {
            self.data.borrow_mut().meta_action = MetaAction::from(meta_action);
            Ok(())
        }

        /* Metas functions */
        #[pymethod]
        pub fn get_metas(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            match self.data.borrow().metas.clone() {
                Some(data) => {
                    let dict = vm.ctx.new_dict();
                    
                    for (key, value) in data.iter() {
                        dict.set_item(key, MetaTypeUtil::as_python_value(value, vm)?, vm)?;
                    }

                    Ok(dict.into())
                },
                None => Ok(vm.ctx.none())
            }
        }

        #[pymethod]
        pub fn set_metas(&self, metas: Option<PyObjectRef>, vm: &VirtualMachine) -> PyResult<()> {
            self.data.borrow_mut().metas = match metas {
                Some(metas) => {
                    
                    let mut new_metas = HashMap::new();

                    if metas.class().fast_issubclass(vm.ctx.types.dict_type) {
                        let dict = metas.downcast_ref::<PyDict>().unwrap();

                        for (key, value) in dict {
                            let key = match key.downcast_ref::<PyStr>() {
                                Some(str) => str.as_str().to_string(),
                                None => return Err(vm.new_exception_msg(PyYummyValidationError::make_class(&vm.ctx), "Only str type allowed for the key.".to_string()))
                            };
                            
                            new_metas.insert(key, MetaTypeUtil::parse_user_meta(vm, &value, UserMetaAccess::User)?.data);
                        }
                    } else {
                        return Err(vm.new_exception_msg(PyYummyValidationError::make_class(&vm.ctx), "Only dict type allowed. .".to_string()))
                    }

                    Some(new_metas)
                },
                None => None
            };
            Ok(())
        }
    }

    #[yummy_model(class_name="DeviceIdAuthRequest")]
    #[pyclass(flags(BASETYPE))]
    impl DeviceIdAuthRequestWrapper {
        #[pymethod]
        pub fn get_device_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_string!(self, id, vm)
        }

        #[pymethod]
        pub fn set_device_id(&self, device_id: String) -> PyResult<()> {
            set_value!(self, id, device_id);
            Ok(())
        }
    }

    #[yummy_model(class_name="EmailAuthRequest")]
    #[pyclass(flags(BASETYPE))]
    impl EmailAuthRequestWrapper {

        #[pymethod]
        pub fn get_email(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_string!(self, email, vm)
        }

        #[pymethod]
        pub fn set_email(&self, email: String) -> PyResult<()> {
            set_value!(self, email, email);
            Ok(())
        }

        #[pymethod]
        pub fn get_password(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_str(&self.data.borrow().password.get()[..]).into())
        }

        #[pymethod]
        pub fn set_password(&self, password: String) -> PyResult<()> {
            self.data.borrow_mut().password = Password::from(password);
            Ok(())
        }

        #[pymethod]
        pub fn get_if_not_exist_create(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_bool(self.data.borrow().if_not_exist_create).into())
        }

        #[pymethod]
        pub fn set_if_not_exist_create(&self, if_not_exist_create: bool) -> PyResult<()> {
            self.data.borrow_mut().if_not_exist_create = if_not_exist_create;
            Ok(())
        }
    }

    #[yummy_model(class_name="CustomIdAuthRequest")]
    #[pyclass(flags(BASETYPE))]
    impl CustomIdAuthRequestWrapper {
        #[pymethod]
        pub fn get_custom_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_string!(self, id, vm)
        }

        #[pymethod]
        pub fn set_custom_id(&self, device_id: String) -> PyResult<()> {
            set_value!(self, id, device_id);
            Ok(())
        }
    }

    #[yummy_model(class_name="CreateRoomRequest")]
    #[pyclass(flags(BASETYPE))]
    impl CreateRoomRequestWrapper {
        /* Name functions */
        #[pymethod]
        pub fn get_name(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_nullable_string!(self, name, vm)
        }

        #[pymethod]
        pub fn set_name(&self, name: Option<String>) -> PyResult<()> {
            set_value!(self, name, name);
            Ok(())
        }

        /* Description functions */
        #[pymethod]
        pub fn get_description(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_nullable_string!(self, description, vm)
        }

        #[pymethod]
        pub fn set_description(&self, description: Option<String>) -> PyResult<()> {
            set_value!(self, description, description);
            Ok(())
        }

        /* Join request functions */
        #[pymethod]
        pub fn get_join_request(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_bool!(self, join_request, vm)
        }

        #[pymethod]
        pub fn set_join_request(&self, join_request: bool) -> PyResult<()> {
            set_value!(self, join_request, join_request);
            Ok(())
        }

        /* Access type functions */
        #[pymethod]
        pub fn get_access_type(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_bigint(&i32::from(self.data.borrow().access_type).to_bigint().unwrap()).into())
        }

        #[pymethod]
        pub fn set_access_type(&self, access_type: CreateRoomAccessTypeWrapper) -> PyResult<()> {
            set_value!(self, access_type, access_type.data);
            Ok(())
        }

        /* Max user functions */
        #[pymethod]
        pub fn get_max_user(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_usize!(self, max_user, vm)
        }

        #[pymethod]
        pub fn set_max_user(&self, max_user: usize) -> PyResult<()> {
            set_value!(self, max_user, max_user);
            Ok(())
        }

        /* Metas functions */
        #[pymethod]
        pub fn get_metas(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            match self.data.borrow().metas.clone() {
                Some(data) => {
                    let dict = vm.ctx.new_dict();
                    
                    for (key, value) in data.iter() {
                        dict.set_item(key, MetaTypeUtil::as_python_value(value, vm)?, vm)?;
                    }

                    Ok(dict.into())
                },
                None => Ok(vm.ctx.none())
            }
        }

        #[pymethod]
        pub fn set_metas(&self, metas: Option<PyObjectRef>, vm: &VirtualMachine) -> PyResult<()> {
            self.data.borrow_mut().metas = match metas {
                Some(metas) => {
                    
                    let mut new_metas = HashMap::new();

                    if metas.class().fast_issubclass(vm.ctx.types.dict_type) {
                        let dict = metas.downcast_ref::<PyDict>().unwrap();

                        for (key, value) in dict {
                            let key = match key.downcast_ref::<PyStr>() {
                                Some(str) => str.as_str().to_string(),
                                None => return Err(vm.new_exception_msg(PyYummyValidationError::make_class(&vm.ctx), "Only str type allowed for the key.".to_string()))
                            };
                            
                            new_metas.insert(key, MetaTypeUtil::parse_room_meta(vm, &value, RoomMetaAccess::default())?.data);
                        }
                    } else {
                        return Err(vm.new_exception_msg(PyYummyValidationError::make_class(&vm.ctx), "Only dict type allowed. .".to_string()))
                    }

                    Some(new_metas)
                },
                None => None
            };
            Ok(())
        }

        /* Tags functions */
        #[pymethod]
        pub fn get_tags(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            let mut list = Vec::new();
            
            for value in self.data.borrow().tags.clone().into_iter() {
                list.push(vm.ctx.new_str(&value[..]).into());
            }

            Ok(vm.ctx.new_list(list).into())
        }

        #[pymethod]
        pub fn set_tags(&self, tags: Vec<PyObjectRef>, vm: &VirtualMachine) -> PyResult<()> {
            let mut new_tags = Vec::new();

            for tag in tags {
                if tag.class().fast_issubclass(vm.ctx.types.str_type) {
                    new_tags.push(tag.payload::<PyStr>().unwrap().as_str().to_string());
                } else {
                    return Err(vm.new_exception_msg(PyYummyValidationError::make_class(&vm.ctx), "Only string type allowed. .".to_string()))
                }
            }
            
            self.data.borrow_mut().tags = new_tags;
            Ok(())
        }
    }

    #[yummy_model(class_name="UpdateRoom")]
    #[pyclass(flags(BASETYPE))]
    impl UpdateRoomWrapper {
        /* RoomId functions */
        #[pymethod]
        pub fn get_room_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_str(&self.data.borrow().room_id.to_string()[..]).into())
        }

        /* Name functions */
        #[pymethod]
        pub fn get_name(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_nullable_string!(self, name, vm)
        }

        #[pymethod]
        pub fn set_name(&self, name: Option<String>) -> PyResult<()> {
            set_value!(self, name, name);
            Ok(())
        }

        /* Description functions */
        #[pymethod]
        pub fn get_description(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_nullable_string!(self, description, vm)
        }

        #[pymethod]
        pub fn set_description(&self, description: Option<String>) -> PyResult<()> {
            set_value!(self, description, description);
            Ok(())
        }

        /* Join request functions */
        #[pymethod]
        pub fn get_join_request(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_nullable_bool!(self, join_request, vm)
        }

        #[pymethod]
        pub fn set_join_request(&self, join_request: Option<bool>) -> PyResult<()> {
            self.data.borrow_mut().join_request = join_request;
            Ok(())
        }

        /* Access type functions */
        #[pymethod]
        pub fn get_access_type(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            match self.data.borrow().access_type {
                Some(access_type) => Ok(vm.ctx.new_bigint(&i32::from(access_type).to_bigint().unwrap()).into()),
                None => Ok(vm.ctx.none())
            }
        }

        #[pymethod]
        pub fn set_access_type(&self, access_type: Option<CreateRoomAccessTypeWrapper>) -> PyResult<()> {
            set_value!(self, access_type, access_type.map(|item| item.data));
            Ok(())
        }

        /* Max user functions */
        #[pymethod]
        pub fn get_max_user(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_nullable_usize!(self, max_user, vm)
        }

        #[pymethod]
        pub fn set_max_user(&self, max_user: Option<usize>) -> PyResult<()> {
            set_value!(self, max_user, max_user);
            Ok(())
        }

        /* Metas functions */
        #[pymethod]
        pub fn get_metas(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            match self.data.borrow().metas.clone() {
                Some(data) => {
                    let dict = vm.ctx.new_dict();
                    
                    for (key, value) in data.iter() {
                        dict.set_item(key, MetaTypeUtil::as_python_value(value, vm)?, vm)?;
                    }

                    Ok(dict.into())
                },
                None => Ok(vm.ctx.none())
            }
        }

        #[pymethod]
        pub fn set_metas(&self, metas: Option<PyObjectRef>, vm: &VirtualMachine) -> PyResult<()> {
            self.data.borrow_mut().metas = match metas {
                Some(metas) => {
                    
                    let mut new_metas = HashMap::new();

                    if metas.class().fast_issubclass(vm.ctx.types.dict_type) {
                        let dict = metas.downcast_ref::<PyDict>().unwrap();

                        for (key, value) in dict {
                            let key = match key.downcast_ref::<PyStr>() {
                                Some(str) => str.as_str().to_string(),
                                None => return Err(vm.new_exception_msg(PyYummyValidationError::make_class(&vm.ctx), "Only str type allowed for the key.".to_string()))
                            };
                            
                            new_metas.insert(key, MetaTypeUtil::parse_room_meta(vm, &value, RoomMetaAccess::default())?.data);
                        }
                    } else {
                        return Err(vm.new_exception_msg(PyYummyValidationError::make_class(&vm.ctx), "Only dict type allowed. .".to_string()))
                    }

                    Some(new_metas)
                },
                None => None
            };
            Ok(())
        }

        /* User permissions functions */
        #[pymethod]
        pub fn get_user_permission(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            match self.data.borrow().user_permission.clone() {
                Some(user_permission) => {
                    let dict = vm.ctx.new_dict();
                    
                    for (key, value) in user_permission.iter() {
                        dict.set_item(&key.to_string(), vm.ctx.new_bigint(&(value.clone() as u32).to_bigint().unwrap()).into(), vm)?;
                    }

                    Ok(dict.into())
                },
                None => Ok(vm.ctx.none())
            }
        }

        #[pymethod]
        pub fn set_user_permission(&self, user_permission: Option<PyObjectRef>, vm: &VirtualMachine) -> PyResult<()> {
            self.data.borrow_mut().user_permission = match user_permission {
                Some(metas) => {
                    let mut new_user_permission = HashMap::new();

                    if metas.class().fast_issubclass(vm.ctx.types.dict_type) {
                        let dict = metas.downcast_ref::<PyDict>().unwrap();

                        for (key, value) in dict {
                            let key = match key.downcast_ref::<PyStr>() {
                                Some(str) => str.as_str().to_string(),
                                None => return Err(vm.new_exception_msg(PyYummyValidationError::make_class(&vm.ctx), "Only str type allowed for the key.".to_string()))
                            };

                            let key = UserId::from(key);
                            new_user_permission.insert(key, RoomUserType::from(value.payload::<PyInt>().unwrap().as_u32_mask() as i32));
                        }
                    } else {
                        return Err(vm.new_exception_msg(PyYummyValidationError::make_class(&vm.ctx), "Only dict type allowed. .".to_string()))
                    }

                    Some(new_user_permission)
                },
                None => None
            };
            Ok(())
        }

        /* Tags functions */
        #[pymethod]
        pub fn get_tags(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            let mut list = Vec::new();
            match self.data.borrow().tags.as_ref() {
                Some(tags) => {
                    for value in tags.clone().into_iter() {
                        list.push(vm.ctx.new_str(&value[..]).into());
                    }
        
                    Ok(vm.ctx.new_list(list).into())
                },
                None => Ok(vm.ctx.none())
            }
        }

        #[pymethod]
        pub fn set_tags(&self, tags: Option<Vec<String>>) -> PyResult<()> {
            self.data.borrow_mut().tags = tags;            
            Ok(())
        }

        /* MetaAction functions */
        #[pymethod]
        pub fn get_meta_action(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_bigint(&(self.data.borrow().meta_action.clone() as u32).to_bigint().unwrap()).into())
        }

        #[pymethod]
        pub fn set_meta_action(&self, meta_action: i32) -> PyResult<()> {
            self.data.borrow_mut().meta_action = MetaAction::from(meta_action);
            Ok(())
        }
    }

    #[yummy_model(class_name="JoinToRoomRequest")]
    #[pyclass(flags(BASETYPE))]
    impl JoinToRoomRequestWrapper {
        /* Room functions */
        #[pymethod]
        pub fn get_room_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_str(&self.data.borrow().room_id.to_string()[..]).into())
        }
        
        /* Room user type functions */
        #[pymethod]
        pub fn get_room_user_type(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_bigint(&(self.data.borrow().room_user_type.clone() as u32).to_bigint().unwrap()).into())
        }
        
        #[pymethod]
        pub fn set_room_user_type(&self, room_user_type: i32) -> PyResult<()> {
            self.data.borrow_mut().room_user_type = RoomUserType::from(room_user_type);
            Ok(())
        }
    }

    #[yummy_model(class_name="ProcessWaitingUser")]
    #[pyclass(flags(BASETYPE))]
    impl ProcessWaitingUserWrapper {
        /* Room function */
        #[pymethod]
        pub fn get_room_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_str(&self.data.borrow().room_id.to_string()[..]).into())
        }
        
        /* User function */
        #[pymethod]
        pub fn get_target_user_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_str(&self.data.borrow().user_id.to_string()[..]).into())
        }

        /* Status functions */
        #[pymethod]
        pub fn get_status(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_bool(self.data.borrow().status).into())
        }

        #[pymethod]
        pub fn set_status(&self, status: bool) -> PyResult<()> {
            self.data.borrow_mut().status = status;
            Ok(())
        }
    }

    #[yummy_model(class_name="KickUserFromRoom")]
    #[pyclass(flags(BASETYPE))]
    impl KickUserFromRoomWrapper {
        /* Room function */
        #[pymethod]
        pub fn get_room_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_str(&self.data.borrow().room_id.to_string()[..]).into())
        }
        
        /* User function */
        #[pymethod]
        pub fn get_target_user_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_str(&self.data.borrow().user_id.to_string()[..]).into())
        }

        /* Ban functions */
        #[pymethod]
        pub fn get_ban(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_bool(self.data.borrow().ban).into())
        }

        #[pymethod]
        pub fn set_ban(&self, status: bool) -> PyResult<()> {
            self.data.borrow_mut().ban = status;
            Ok(())
        }
    }

    #[yummy_model(class_name="DisconnectFromRoomRequest")]
    #[pyclass(flags(BASETYPE))]
    impl DisconnectFromRoomRequestWrapper {
        /* Room function */
        #[pymethod]
        pub fn get_room_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_str(&self.data.borrow().room_id.to_string()[..]).into())
        }
    }

    #[yummy_model(class_name="MessageToRoomRequest")]
    #[pyclass(flags(BASETYPE))]
    impl MessageToRoomRequestWrapper {
        /* Room function */
        #[pymethod]
        pub fn get_room_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_str(&self.data.borrow().room_id.to_string()[..]).into())
        }

        /* Message function */
        #[pymethod]
        pub fn get_message(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            match py_serde::deserialize(vm, self.data.borrow().message.clone()) {
                Ok(message) => Ok(message),
                Err(error) => Err(vm.new_exception_msg(PyYummyValidationError::make_class(&vm.ctx), error.to_string()))
            }
        }

        #[pymethod]
        pub fn set_message(&self, message: PyObjectRef, vm: &VirtualMachine) -> PyResult<()> {
            let obj_serializer = rustpython_vm::py_serde::PyObjectSerializer::new(vm, &message);
            self.data.borrow_mut().message = match serde_json::value::to_value(obj_serializer) {
                Ok(message) => message,
                Err(error) => return Err(vm.new_exception_msg(PyYummyValidationError::make_class(&vm.ctx), error.to_string()))
            };
            Ok(())
        }
    }

    #[yummy_model(class_name="RoomListRequest", no_auth=true)]
    #[pyclass(flags(BASETYPE))]
    impl RoomListRequestWrapper {
        #[pymethod]
        pub fn get_tag(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            get_nullable_string!(self, tag, vm)
        }

        #[pymethod]
        pub fn set_tag(&self, tag: Option<String>) -> PyResult<()> {
            set_value!(self, tag, tag);
            Ok(())
        }

        #[pymethod]
        pub fn get_members(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            let mut list = Vec::new();
            
            for value in self.data.borrow().members.clone().into_iter() {
                list.push(vm.ctx.new_bigint(&(<RoomInfoTypeVariant as Into<u32>>::into(value).to_bigint().unwrap())).into());
            }

            Ok(vm.ctx.new_list(list).into())
        }

        #[pymethod]
        pub fn set_members(&self, members: Vec<PyObjectRef>, vm: &VirtualMachine) -> PyResult<()> {
            let mut new_members = Vec::new();

            for member in members {
                if member.class().fast_issubclass(vm.ctx.types.int_type) {
                    new_members.push(member.payload::<PyInt>().unwrap().as_u32_mask().into());
                } else {
                    return Err(vm.new_exception_msg(PyYummyValidationError::make_class(&vm.ctx), "Only int type allowed.".to_string()))
                }
            }
            
            self.data.borrow_mut().members = new_members;
            Ok(())
        }
    }

    #[yummy_model(class_name="WaitingRoomJoins")]
    #[pyclass(flags(BASETYPE))]
    impl WaitingRoomJoinsWrapper {
        #[pymethod]
        pub fn get_room_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_str(&self.data.borrow().room_id.to_string()[..]).into())
        }
    }

    #[yummy_model(class_name="GetRoomRequest")]
    #[pyclass(flags(BASETYPE))]
    impl GetRoomRequestWrapper {
        #[pymethod]
        pub fn get_room_id(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_str(&self.data.borrow().room_id.to_string()[..]).into())
        }

        #[pymethod]
        pub fn get_members(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            let mut list = Vec::new();
            
            for value in self.data.borrow().members.clone().into_iter() {
                list.push(vm.ctx.new_bigint(&(<RoomInfoTypeVariant as Into<u32>>::into(value).to_bigint().unwrap())).into());
            }

            Ok(vm.ctx.new_list(list).into())
        }

        #[pymethod]
        pub fn set_members(&self, members: Vec<PyObjectRef>, vm: &VirtualMachine) -> PyResult<()> {
            let mut new_members = Vec::new();

            for member in members {
                if member.class().fast_issubclass(vm.ctx.types.int_type) {
                    new_members.push(member.payload::<PyInt>().unwrap().as_u32_mask().into());
                } else {
                    return Err(vm.new_exception_msg(PyYummyValidationError::make_class(&vm.ctx), "Only int type allowed. .".to_string()))
                }
            }
            
            self.data.borrow_mut().members = new_members;
            Ok(())
        }
    }

    /* **************************************************************************************************************** */
    /* ********************************************** TRAIT IMPLEMENTS ************************************************ */
    /* **************************************************************************************************************** */

    impl TryFromBorrowedObject for UserMetaAccessWrapper {
        fn try_from_borrowed_object(vm: &VirtualMachine, obj: &PyObject) -> Result<Self, PyRef<PyBaseException>> {
            if obj.class().fast_issubclass(vm.ctx.types.int_type) {
                return Ok(UserMetaAccessWrapper::new(UserMetaAccess::from(obj.payload::<PyInt>().unwrap().as_u32_mask() as i32)));
            }

            Ok(UserMetaAccessWrapper { data: UserMetaAccess::System })
        }
    }

    impl TryFromBorrowedObject for RoomMetaAccessWrapper {
        fn try_from_borrowed_object(vm: &VirtualMachine, obj: &PyObject) -> Result<Self, PyRef<PyBaseException>> {
            if obj.class().fast_issubclass(vm.ctx.types.int_type) {
                return Ok(RoomMetaAccessWrapper::new(RoomMetaAccess::from(obj.payload::<PyInt>().unwrap().as_u32_mask() as i32)));
            }

            Ok(RoomMetaAccessWrapper { data: RoomMetaAccess::System })
        }
    }

    impl TryFromBorrowedObject for CreateRoomAccessTypeWrapper {
        fn try_from_borrowed_object(vm: &VirtualMachine, obj: &PyObject) -> Result<Self, PyRef<PyBaseException>> {
            if obj.class().fast_issubclass(vm.ctx.types.int_type) {
                return Ok(CreateRoomAccessTypeWrapper::new(CreateRoomAccessType::from(obj.payload::<PyInt>().unwrap().as_u32_mask() as i32)));
            }

            Ok(CreateRoomAccessTypeWrapper { data: CreateRoomAccessType::default() })
        }
    }

    impl TryFromBorrowedObject for RoomInfoTypeVariantWrapper {
        fn try_from_borrowed_object(vm: &VirtualMachine, obj: &PyObject) -> Result<Self, PyRef<PyBaseException>> {
            if obj.class().fast_issubclass(vm.ctx.types.int_type) {
                return Ok(RoomInfoTypeVariantWrapper::new(obj.payload::<PyInt>().unwrap().as_u32_mask().into()));
            }

            panic!("");
        }
    }

    /* **************************************************************************************************************** */
    /* ************************************************* MACROS CALL ************************************************** */
    /* ************************************************** UNIT TESTS ************************************************** */
    /* **************************************************************************************************************** */
}
