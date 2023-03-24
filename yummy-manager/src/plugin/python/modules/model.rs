#[rustpython::vm::pymodule]
pub mod _model {
    /* **************************************************************************************************************** */
    /* **************************************************** MODS ****************************************************** */
    /* *************************************************** IMPORTS **************************************************** */
    /* **************************************************************************************************************** */
    use std::{rc::Rc, cell::RefCell};
    use std::ops::Deref;
    use std::collections::HashMap;
    
    use num_bigint::ToBigInt;

    use rustpython_vm::builtins::{PyBaseException, PyInt, PyDict, PyStr};
    use rustpython_vm::{VirtualMachine, PyResult, PyObjectRef, TryFromBorrowedObject, PyRef, PyObject};
    use yummy_general::password::Password;
    use yummy_model::UserType;
    use yummy_model::meta::MetaAction;
    use yummy_model::{meta::{UserMetaAccess, UserMetaType}};
    use yummy_macros::yummy_model;
    use rustpython::vm::{pyclass, PyPayload};
    use rustpython::vm::builtins::PyIntRef;
    use rustpython_vm::class::PyClassImpl;

    use crate::plugin::python::modules::base::_base::PyYummyValidationError;
    use crate::plugin::python::util::MetaTypeUtil;
    use crate::{auth::model::{DeviceIdAuthRequest, EmailAuthRequest, CustomIdAuthRequest, ConnUserDisconnect, LogoutRequest, RefreshTokenRequest, RestoreTokenRequest}, conn::model::UserConnected, user::model::{UpdateUser, GetUserInformation, GetUserInformationEnum}, room::model::CreateRoomRequest};
    use crate::plugin::python::ModelWrapper;

    /* **************************************************************************************************************** */
    /* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
    /* **************************************************** MACROS **************************************************** */
    /* **************************************************************************************************************** */

    macro_rules! wrapper_struct {
        ($model: ident, $wrapper: ident, $class_name: expr) => {
            #[pyclass(module = "yummy", name = $class_name)]
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

    macro_rules! get_string {
        ($self: expr, $item: ident, $vm: ident) => {
            Ok($vm.ctx.new_str(&$self.data.borrow_mut().$item[..]).into())
        };
    }

    macro_rules! get_bool {
        ($self: expr, $item: ident, $vm: ident) => {
            Ok($vm.ctx.new_bool($self.data.borrow_mut().$item).into())
        };
    }

    macro_rules! set_value {
        ($self: expr,  $target: ident, $source: ident) => {
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
                None => Ok($vm.ctx.none().into())
            }
        };
    }

    macro_rules! get_nullable_string {
        ($self: expr, $target: ident, $vm: ident) => {
            match &$self.data.borrow().$target {
                Some(data) => Ok($vm.ctx.new_str(&data.to_string()[..]).into()),
                None => Ok($vm.ctx.none().into())
            }
        };
    }

    macro_rules! get_user_id {
        ($self: expr, $vm: ident) => {
            match $self.data.borrow().auth.deref() {
                Some(auth) => Ok($vm.ctx.new_str(&auth.user.to_string()[..]).into()),
                None => Ok($vm.ctx.none().into())
            }
        };
    }

    macro_rules! get_session_id {
        ($self: expr, $vm: ident) => {
            match $self.data.borrow().auth.deref() {
                Some(auth) => Ok($vm.ctx.new_str(&auth.session.to_string()[..]).into()),
                None => Ok($vm.ctx.none().into())
            }
        };
    }

    /* **************************************************************************************************************** */
    /* *************************************************** STRUCTS **************************************************** */
    /* **************************************************************************************************************** */
    wrapper_struct!(DeviceIdAuthRequest, DeviceIdAuthRequestWrapper, "DeviceIdAuth");
    wrapper_struct!(EmailAuthRequest, EmailAuthRequestWrapper, "EmailAuth");
    wrapper_struct!(CustomIdAuthRequest, CustomIdAuthRequestWrapper, "CustomIdAuth");
    wrapper_struct!(UserConnected, UserConnectedWrapper, "UserConnected");
    wrapper_struct!(ConnUserDisconnect, ConnUserDisconnectWrapper, "ConnUserDisconnect");
    wrapper_struct!(UpdateUser, UpdateUserWrapper, "UpdateUser");
    wrapper_struct!(LogoutRequest, LogoutRequestWrapper, "Logout");
    wrapper_struct!(RefreshTokenRequest, RefreshTokenRequestWrapper, "RefreshToken");
    wrapper_struct!(RestoreTokenRequest, RestoreTokenRequestWrapper, "RestoreToken");
    wrapper_struct!(GetUserInformation, GetUserInformationWrapper, "GetUserInformation");
    wrapper_struct!(CreateRoomRequest, CreateRoomRequestWrapper, "CreateRoom");

    #[pyattr]
    #[pyclass(module = false, name = "UserMetaType")]
    #[derive(Debug, PyPayload)]
    pub struct UserMetaTypeWrapper {
        pub data: UserMetaType
    }

    #[pyattr]
    #[pyclass(module = false, name = "UserMetaAccess")]
    #[derive(Debug, PyPayload)]
    pub struct UserMetaAccessWrapper {
        pub data: UserMetaAccess
    }

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
            Ok(vm.ctx.new_str(&self.data.borrow_mut().user_id.to_string()[..]).into())
        }
    }

    #[yummy_model(class_name="RefreshTokenRequest")]
    #[pyclass(flags(BASETYPE))]
    impl RefreshTokenRequestWrapper {
        #[pymethod]
        pub fn get_token(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_str(&self.data.borrow_mut().token.to_string()[..]).into())
        }
    }

    #[yummy_model(class_name="RestoreTokenRequest")]
    #[pyclass(flags(BASETYPE))]
    impl RestoreTokenRequestWrapper {
        #[pymethod]
        pub fn get_token(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_str(&self.data.borrow_mut().token.to_string()[..]).into())
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
                None => Ok(vm.ctx.none().into())
            }
        }

        #[pymethod]
        pub fn set_user_type(&self, user_type: Option<i32>) -> PyResult<()> {
            self.data.borrow_mut().user_type = user_type.map(|item| UserType::from(item));
            Ok(())
        }

        /* MetaAction functions */
        #[pymethod]
        pub fn get_meta_action(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            match self.data.borrow().meta_action.clone() {
                Some(data) => Ok(vm.ctx.new_bigint(&(data as u32).to_bigint().unwrap()).into()),
                None => Ok(vm.ctx.none().into())
            }
        }

        #[pymethod]
        pub fn set_meta_action(&self, meta_action: Option<i32>) -> PyResult<()> {
            self.data.borrow_mut().meta_action = meta_action.map(|item| MetaAction::from(item));
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
                None => Ok(vm.ctx.none().into())
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
                            
                            new_metas.insert(key, MetaTypeUtil::parse(vm, &value, UserMetaAccess::User)?.data);
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
            Ok(vm.ctx.new_str(&self.data.borrow_mut().password.get()[..]).into())
        }

        #[pymethod]
        pub fn set_password(&self, password: String) -> PyResult<()> {
            self.data.borrow_mut().password = Password::from(password);
            Ok(())
        }

        #[pymethod]
        pub fn get_if_not_exist_create(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            Ok(vm.ctx.new_bool(self.data.borrow_mut().if_not_exist_create).into())
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
    impl CreateRoomRequestWrapper { }

    /* ########################################### UserMetaTypeWrapper ################################################# */
    #[pyclass(flags(BASETYPE))]
    impl UserMetaTypeWrapper {
        pub fn new(data: UserMetaType) -> Self {
            Self { data }
        }
    }

    #[pyclass(flags(BASETYPE))]
    impl UserMetaAccessWrapper {
        pub fn new(data: UserMetaAccess) -> Self {
            Self { data }
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

    /* **************************************************************************************************************** */
    /* ************************************************* MACROS CALL ************************************************** */
    /* ************************************************** UNIT TESTS ************************************************** */
    /* **************************************************************************************************************** */
}
