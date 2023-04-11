use std::collections::HashMap;
/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::{sync::Arc, env::temp_dir};
use std::io::Write;

use yummy_cache::state::{YummyState, RoomInfoTypeVariant};
use yummy_general::password::Password;
use yummy_model::meta::{MetaAction, UserMetaType, UserMetaAccess, RoomMetaAccess, RoomMetaType};
use yummy_model::{UserId, SessionId, UserType, CreateRoomAccessType, RoomId, RoomUserType};
use yummy_model::auth::UserAuth;
use yummy_model::config::YummyConfig;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use yummy_testing::cache::DummyResourceFactory;
use yummy_testing::client::DummyClient;
use yummy_testing::database::get_database_pool;

use crate::auth::model::{EmailAuthRequest, CustomIdAuthRequest, LogoutRequest, RefreshTokenRequest, RestoreTokenRequest, ConnUserDisconnect};
use crate::conn::model::UserConnected;
use crate::plugin::PluginExecuter;
use crate::room::model::{CreateRoomRequest, UpdateRoom, JoinToRoomRequest, ProcessWaitingUser, KickUserFromRoom, DisconnectFromRoomRequest, MessageToRoomRequest, RoomListRequest, WaitingRoomJoins, GetRoomRequest};
use crate::user::model::{GetUserInformation, GetUserInformationEnum, UpdateUser};
use crate::{plugin::{PluginBuilder}, auth::model::{DeviceIdAuthRequest}};
use super::PythonPluginInstaller;

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* **************************************************************************************************************** */
macro_rules! model_tester {
    ($name: ident, $file_name: expr, $pre_func: ident, $post_func: ident, $model: expr) => {
        
        #[test]
        fn $name() {
            let code = r#"
def pre_func(model):
    assert(model.get_user_id()    == "294a6097-b8ea-4daa-b699-9f0c0c119c6d")
    assert(model.get_session_id() == "1bca52a9-4b98-45dd-bda9-93468d1b583f")
    assert(model.get_request_id() == 123)
    
    model.set_request_id(None)
    assert(model.get_request_id() is None)

    model.set_request_id(123)
    assert(model.get_request_id() == 123)

def post_func(model, success):
    assert(model.get_user_id()    == "294a6097-b8ea-4daa-b699-9f0c0c119c6d")
    assert(model.get_session_id() == "1bca52a9-4b98-45dd-bda9-93468d1b583f")
    assert(model.get_request_id() == 123)
        
    model.set_request_id(None)
    assert(model.get_request_id() is None)

    model.set_request_id(123)
    assert(model.get_request_id() == 123)
"#;

            let code = code
                .replace("pre_func", stringify!($pre_func))
                .replace("post_func", stringify!($post_func));

            let (executer, _) = create_python_environtment($file_name, &code);

            let model = executer. $pre_func  ($model).expect("pre_deviceid_auth returned Err");
            executer. $post_func (model, true).expect("post_deviceid_auth returned Err");
        }
    };
}

/* **************************************************************************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* **************************************************************************************************************** */
fn create_python_file(file_name: &str, config: &mut YummyConfig, content: &str) {

    // Generate random folder name
    let folder_name: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    config.python_files_path = temp_dir().join(folder_name).to_str().expect("Could not get temporary folder path").to_string();

    std::fs::create_dir(&config.python_files_path).expect("Could not create temporary folder for python test"); // Create temp folder

    let path = std::path::Path::new(&config.python_files_path[..]).join(file_name).to_string_lossy().to_string();

    let mut python_file = std::fs::File::create(&path).expect("create failed");
    python_file.write_all(content.as_bytes()).expect("write failed");
}

fn create_python_environtment(file_name: &str, content: &str) -> (Arc<PluginExecuter>, YummyState) {
    let mut config = YummyConfig::default();
    create_python_file(file_name, &mut config, content);

    let config = Arc::new(config);
    let mut builder = PluginBuilder::default();
    builder.add_installer(Box::new(PythonPluginInstaller::default()));

    
    let connection = get_database_pool();
    let state = YummyState::new(config.clone(), Box::new(DummyResourceFactory{}));
    (Arc::new(builder.build(config, state.clone(), connection)), state)
}

/* **************************************************************************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */

#[test]
fn simple_python_api_call() {
    create_python_environtment("simple_python_api_call.py", r#"
def test(model: dict):
    print("Merhaba dünya")
    return 123
"#);
}

#[test]
fn enum_types() {
    create_python_environtment("simple_python_api_call.py", r#"
from yummy import constants

# UserType's
constants.USER_TYPE_USER
constants.USER_TYPE_MOD
constants.USER_TYPE_ADMIN

# UserMetaAccess's
constants.USER_META_ACCESS_ANONYMOUS
constants.USER_META_ACCESS_USER
constants.USER_META_ACCESS_FRIEND
constants.USER_META_ACCESS_ME
constants.USER_META_ACCESS_MOD
constants.USER_META_ACCESS_ADMIN
constants.USER_META_ACCESS_SYSTEM

# MetaAction
constants.META_ACTION_ONLY_ADD_OR_UPDATE
constants.META_ACTION_REMOVE_UNUSED_METAS
constants.META_ACTION_REMOVE_ALL_METAS

# CreateRoomAccessType
constants.ROOM_ACCESS_TYPE_PUBLIC
constants.ROOM_ACCESS_TYPE_PRIVATE
constants.ROOM_ACCESS_TYPE_FRIEND

# RoomUserType
constants.ROOM_USER_TYPE_USER
constants.ROOM_USER_TYPE_MODERATOR
constants.ROOM_USER_TYPE_OWNER

# RoomInfoType
constants.ROOM_INFO_TYPE_ROOM_NAME
constants.ROOM_INFO_TYPE_DESCRIPTION
constants.ROOM_INFO_TYPE_USERS
constants.ROOM_INFO_TYPE_MAX_USER
constants.ROOM_INFO_TYPE_USER_LENGTH
constants.ROOM_INFO_TYPE_ACCESS_TYPE
constants.ROOM_INFO_TYPE_JOIN_REQUEST
constants.ROOM_INFO_TYPE_INSERT_DATE
constants.ROOM_INFO_TYPE_TAGS
constants.ROOM_INFO_TYPE_BANNED_USERS
constants.ROOM_INFO_TYPE_METAS
"#);
}


#[test]
fn email_auth_test() {
    let (executer, _) = create_python_environtment("email_auth_test.py", r#"
def pre_email_auth(model):
    assert(model.get_email() == "abc@gmail.com")
    model.set_email("abc@abc.com")
    assert(model.get_email() == "abc@abc.com")

    assert(model.get_password() == "password123")
    model.set_password("password")
    assert(model.get_password() == "password")

    assert(model.get_if_not_exist_create() is True)
    model.set_if_not_exist_create(False)
    assert(model.get_if_not_exist_create() is False)

def post_email_auth(model, success):
    assert(success)
    assert(model.get_email() == "abc@abc.com")
    assert(model.get_password() == "password")
    assert(model.get_if_not_exist_create() is False)

"#);

    let model = EmailAuthRequest {
        request_id: Some(123),
        auth: Arc::new(Some(UserAuth {
            user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
            session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
        })),
        email: "abc@gmail.com".to_string(),
        password: Password::from("password123".to_string()),
        if_not_exist_create: true,
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_email_auth(model).expect("pre_email_auth returned Err");
    let model = executer.post_email_auth(model, true).expect("post_email_auth returned Err");
    
    assert_eq!(&model.email[..], "abc@abc.com");
    assert_eq!(&model.password.get()[..], "password");
    assert_eq!(model.if_not_exist_create, false);
}

#[test]
fn deviceid_auth_test() {
    let (executer, _) = create_python_environtment("deviceid_auth_test.py", r#"
def pre_deviceid_auth(model):
    assert(model.get_device_id() == "abc")
    model.set_device_id("erhan")
    assert(model.get_device_id() == "erhan")

    assert(model.get_request_id() == 123)
    model.set_request_id(None)
    assert(model.get_request_id() is None)

def post_deviceid_auth(model, success):
    assert(success)
    assert(model.get_device_id() == "erhan")
    model.set_device_id("abc")
    assert(model.get_device_id() == "abc")

    model.set_device_id("erhan")

    assert(model.get_request_id() is None)
    model.set_request_id(None)
    assert(model.get_request_id() is None)
"#);

    let model = DeviceIdAuthRequest {
        request_id: Some(123),
        auth: Arc::new(None),
        id: "abc".to_string(),
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_deviceid_auth(model).expect("pre_deviceid_auth returned Err");
    let model = executer.post_deviceid_auth(model, true).expect("post_deviceid_auth returned Err");
    
    assert_eq!(&model.id, "erhan");
}

#[test]
fn customid_auth_test() {
    let (executer, _) = create_python_environtment("customid_auth_test.py", r#"
def pre_customid_auth(model):
    assert(model.get_custom_id() == "abc")
    model.set_custom_id("erhan")
    assert(model.get_custom_id() == "erhan")

    assert(model.get_request_id() == 123)
    model.set_request_id(None)
    assert(model.get_request_id() is None)

def post_customid_auth(model, success):
    assert(success)
    assert(model.get_custom_id() == "erhan")
    model.set_custom_id("abc")
    assert(model.get_custom_id() == "abc")

    model.set_custom_id("erhan")

    assert(model.get_request_id() is None)
    model.set_request_id(None)
    assert(model.get_request_id() is None)
"#);

    let model = CustomIdAuthRequest {
        request_id: Some(123),
        auth: Arc::new(None),
        id: "abc".to_string(),
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_customid_auth(model).expect("pre_customid_auth returned Err");
    let model = executer.post_customid_auth(model, true).expect("post_customid_auth returned Err");

    assert_eq!(&model.id, "erhan");
}

#[test]
fn validation_exception_test() {
    let (executer, _) = create_python_environtment("validation_exception_test.py", r#"
import yummy
def pre_deviceid_auth(model):
    yummy.fail("fail")
"#);

    let model = DeviceIdAuthRequest {
        request_id: Some(123),
        auth: Arc::new(None),
        id: "abc".to_string(),
        socket: Arc::new(DummyClient::default())
    };

    let result = executer.pre_deviceid_auth(model);
    if let Err(error) = result {
        println!("{}", error.to_string());
        assert!(error.to_string() == "fail".to_string());
    } else {
        assert!(false, "No python raise received")
    }
}

#[test]
fn user_meta_test() {
    let (executer, _) = create_python_environtment("user_meta_test.py", r#"
import yummy

def pre_deviceid_auth(model):
    assert(yummy.user.get_user_meta(model.get_user_id(), "key") is None)
    
    assert(yummy.user.set_user_meta(model.get_user_id(), "key", "motorola"))
    assert(yummy.user.get_user_meta(model.get_user_id(), "key") == "motorola")

    assert(yummy.user.set_user_meta(model.get_user_id(), "key", 2023))
    assert(yummy.user.get_user_meta(model.get_user_id(), "key") == 2023)
    
    assert(yummy.user.set_user_meta(model.get_user_id(), "key", "motorola", 3))
    assert(yummy.user.get_user_meta(model.get_user_id(), "key") == "motorola")

    assert(yummy.user.set_user_meta(model.get_user_id(), "key", 2023, 1))
    assert(yummy.user.get_user_meta(model.get_user_id(), "key") == 2023)

    assert(yummy.user.set_user_meta(model.get_user_id(), "key", [123, True, 321.123, "test"]))
    assert(yummy.user.get_user_meta(model.get_user_id(), "key") == [123, True, 321.123, "test"])

    assert(yummy.user.set_user_meta(model.get_user_id(), "key", None))
    assert(yummy.user.get_user_meta(model.get_user_id(), "key") is None)

    yummy.user.set_user_meta(model.get_user_id(), "key1", "motorola")
    yummy.user.set_user_meta(model.get_user_id(), "key2", True)
    yummy.user.set_user_meta(model.get_user_id(), "key3", 123)
    yummy.user.set_user_meta(model.get_user_id(), "key4", 321.123)
    yummy.user.set_user_meta(model.get_user_id(), "key5", [123, True, 321.123, "test"])

    assert(yummy.user.get_user_metas(model.get_user_id()) == {'key': None, 'key1': 'motorola', 'key2': True, 'key3': 123.0, 'key4': 321.123, 'key5': [123.0, True, 321.123, 'test']})

    yummy.user.remove_user_meta(model.get_user_id(), "key1")
    assert(yummy.user.get_user_metas(model.get_user_id()) == {'key': None, 'key2': True, 'key3': 123.0, 'key4': 321.123, 'key5': [123.0, True, 321.123, 'test']})

    yummy.user.remove_user_meta(model.get_user_id(), "dummy")
    assert(yummy.user.get_user_metas(model.get_user_id()) == {'key': None, 'key2': True, 'key3': 123.0, 'key4': 321.123, 'key5': [123.0, True, 321.123, 'test']})

    assert(yummy.user.remove_user_metas(model.get_user_id()))
    assert(yummy.user.get_user_metas(model.get_user_id()) == {})
"#);

    let model = DeviceIdAuthRequest {
        request_id: Some(123),
        auth: Arc::new(Some(UserAuth {
            user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
            session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
        })),
        id: "abc".to_string(),
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_deviceid_auth(model).expect("pre_deviceid_auth returned Err");
    executer.post_deviceid_auth(model, true).expect("post_deviceid_auth returned Err");
}

#[test]
fn user_connected_test() {
    let (executer, _) = create_python_environtment("user_connected_test.py", r#"
import yummy

def pre_user_connected(model):
    assert(model.get_user_id() == "294a6097-b8ea-4daa-b699-9f0c0c119c6d")

def post_user_connected(model, success):
    assert(model.get_user_id() == "294a6097-b8ea-4daa-b699-9f0c0c119c6d")
"#);

    let model = UserConnected {
        user_id: Arc::new(UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string())),
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_user_connected(model).expect("pre_user_connected returned Err");
    executer.post_user_connected(model, true).expect("post_user_connected returned Err");
}


#[test]
fn user_disconnected_test() {
    let (executer, _) = create_python_environtment("user_disconnected_test.py", r#"
import yummy

def pre_user_disconnected(model):
    assert(model.get_send_message() is False)
    model.set_send_message(True)

def post_user_disconnected(model, success):
    assert(model.get_send_message())
"#);

    let model = ConnUserDisconnect {
        request_id: Some(123),
        auth: Arc::new(None),
        send_message: false,
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_user_disconnected(model).expect("pre_user_disconnected returned Err");
    let model = executer.post_user_disconnected(model, true).expect("post_user_disconnected returned Err");
    
    assert_eq!(model.send_message, true);
}

#[test]
fn refresh_token_test() {
    let (executer, _) = create_python_environtment("refresh_token_test.py", r#"
import yummy

def pre_refresh_token(model):
    assert(model.get_token() == "TOKEN")

def post_refresh_token(model, success):
    assert(model.get_token() == "TOKEN")
"#);

    let model = RefreshTokenRequest {
        request_id: Some(123),
        auth: Arc::new(Some(UserAuth {
            user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
            session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
        })),
        token: "TOKEN".to_string(),
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_refresh_token(model).expect("pre_refresh_token returned Err");
    executer.post_refresh_token(model, true).expect("post_refresh_token returned Err");
}

#[test]
fn restore_token_test() {
    let (executer, _) = create_python_environtment("restore_token_test.py", r#"
import yummy

def pre_restore_token(model):
    assert(model.get_token() == "TOKEN")

def post_restore_token(model, success):
    assert(model.get_token() == "TOKEN")
"#);

    let model = RestoreTokenRequest {
        request_id: Some(123),
        auth: Arc::new(Some(UserAuth {
            user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
            session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
        })),
        token: "TOKEN".to_string(),
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_restore_token(model).expect("pre_restore_token returned Err");
    executer.post_restore_token(model, true).expect("post_restore_token returned Err");
}

#[test]
fn get_user_information_test() {

    /* Me test 1 */
    let (executer, _) = create_python_environtment("get_user_information_test.py", r#"
import yummy

def pre_get_user_information(model):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "Me")
    assert(model.get_user_id() is None)
    assert(model.get_value() == (None, None))
    assert(model.get_requester_user_id() is None)

def post_get_user_information(model, success):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "Me")
    assert(model.get_user_id() is None)
    assert(model.get_value() == (None, None))
    assert(model.get_requester_user_id() is None)
"#);

    let model = GetUserInformation {
        request_id: Some(123),
        query: GetUserInformationEnum::Me(Arc::new(None)),
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_get_user_information(model).expect("pre_get_user_information returned Err");
    executer.post_get_user_information(model, true).expect("post_get_user_information returned Err");


    /* Me test 2 */
    let (executer, _) = create_python_environtment("get_user_information_test.py", r#"
import yummy

def pre_get_user_information(model):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "Me")
    assert(model.get_user_id() == "294a6097-b8ea-4daa-b699-9f0c0c119c6d")
    assert(model.get_value() == ('294a6097-b8ea-4daa-b699-9f0c0c119c6d', None))

def post_get_user_information(model, success):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "Me")
    assert(model.get_user_id() == "294a6097-b8ea-4daa-b699-9f0c0c119c6d")
    assert(model.get_value() == ('294a6097-b8ea-4daa-b699-9f0c0c119c6d', None))
"#);

    let model = GetUserInformation {
        request_id: Some(123),
        query: GetUserInformationEnum::Me(Arc::new(Some(UserAuth {
            user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
            session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
        }))),
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_get_user_information(model).expect("pre_get_user_information returned Err");
    executer.post_get_user_information(model, true).expect("post_get_user_information returned Err");


    /* UserViaSystem test */
    let (executer, _) = create_python_environtment("get_user_information_test.py", r#"
import yummy

def pre_get_user_information(model):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "UserViaSystem")
    assert(model.get_user_id() == "294a6097-b8ea-4daa-b699-9f0c0c119c6d")
    assert(model.get_value() == ("294a6097-b8ea-4daa-b699-9f0c0c119c6d", None))

def post_get_user_information(model, success):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "UserViaSystem")
    assert(model.get_user_id() == "294a6097-b8ea-4daa-b699-9f0c0c119c6d")
    assert(model.get_value() == ("294a6097-b8ea-4daa-b699-9f0c0c119c6d", None))
"#);

    let model = GetUserInformation {
        request_id: Some(123),
        query: GetUserInformationEnum::UserViaSystem(UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string())),
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_get_user_information(model).expect("pre_get_user_information returned Err");
    executer.post_get_user_information(model, true).expect("post_get_user_information returned Err");


    /* User test 1 */
    let (executer, _) = create_python_environtment("get_user_information_tes2.py", r#"
import yummy

def pre_get_user_information(model):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "User")
    assert(model.get_user_id() == "294a6097-b8ea-4daa-b699-9f0c0c119c6d")
    assert(model.get_value() == ('294a6097-b8ea-4daa-b699-9f0c0c119c6d', '2fc4cc71-b43f-4246-b072-710ad1d2095c'))
    assert(model.get_requester_user_id() == "2fc4cc71-b43f-4246-b072-710ad1d2095c")

def post_get_user_information(model, success):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "User")
    assert(model.get_user_id() == "294a6097-b8ea-4daa-b699-9f0c0c119c6d")
    assert(model.get_value() == ('294a6097-b8ea-4daa-b699-9f0c0c119c6d', '2fc4cc71-b43f-4246-b072-710ad1d2095c'))
    assert(model.get_requester_user_id() == "2fc4cc71-b43f-4246-b072-710ad1d2095c")
"#);

    let model = GetUserInformation {
        request_id: Some(123),
        query: GetUserInformationEnum::User {
            user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
            requester: Arc::new(Some(UserAuth {
                user: UserId::from("2fc4cc71-b43f-4246-b072-710ad1d2095c".to_string()),
                session: SessionId::from("6b8022d8-8eec-48e1-90f8-edb8fa637761".to_string())
            }))
        },
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_get_user_information(model).expect("pre_get_user_information returned Err");
    executer.post_get_user_information(model, true).expect("post_get_user_information returned Err");


    /* User test 2 */
    let (executer, _) = create_python_environtment("get_user_information_test.py", r#"
import yummy

def pre_get_user_information(model):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "User")
    assert(model.get_user_id() == "294a6097-b8ea-4daa-b699-9f0c0c119c6d")
    assert(model.get_value() == ("294a6097-b8ea-4daa-b699-9f0c0c119c6d", "edbe3335-3545-4d38-a2f6-3856e63bfd6f"))

def post_get_user_information(model, success):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "User")
    assert(model.get_user_id() == "294a6097-b8ea-4daa-b699-9f0c0c119c6d")
    assert(model.get_value() == ("294a6097-b8ea-4daa-b699-9f0c0c119c6d", "edbe3335-3545-4d38-a2f6-3856e63bfd6f"))
"#);

    let model = GetUserInformation {
        request_id: Some(123),
        query: GetUserInformationEnum::User {
            user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
            requester: Arc::new(Some(UserAuth {
                user: UserId::from("edbe3335-3545-4d38-a2f6-3856e63bfd6f".to_string()),
                session: SessionId::from("69531fc4-bb09-41a7-aeb0-364876f1ff79".to_string())
            }))
        },
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_get_user_information(model).expect("pre_get_user_information returned Err");
    executer.post_get_user_information(model, true).expect("post_get_user_information returned Err");
}

#[test]
fn user_update_test() {

    /* Get all update fields */
    let (executer, _) = create_python_environtment("user_update_test1.py", r#"
import yummy

def pre_update_user(model):
    assert(model.get_request_id() == 123)
    assert(model.get_target_user_id() == "1ea7b016-fdd2-4d07-b71c-f877049265da")
    assert(model.get_name() == "erhan")
    assert(model.get_password() == "abc")
    assert(model.get_email() == "erhan@abc.com")
    assert(model.get_device_id() == "device_id")
    assert(model.get_custom_id() == "custom_id")
    assert(model.get_user_type() == yummy.constants.USER_TYPE_ADMIN)
    assert(model.get_meta_action() == yummy.constants.META_ACTION_REMOVE_ALL_METAS)
    assert(model.get_metas() == {})

def post_update_user(model, success):
    assert(model.get_request_id() == 123)
    assert(model.get_target_user_id() == "1ea7b016-fdd2-4d07-b71c-f877049265da")
    assert(model.get_name() == "erhan")
    assert(model.get_password() == "abc")
    assert(model.get_email() == "erhan@abc.com")
    assert(model.get_device_id() == "device_id")
    assert(model.get_custom_id() == "custom_id")
    assert(model.get_user_type() == yummy.constants.USER_TYPE_ADMIN)
    assert(model.get_meta_action() == yummy.constants.META_ACTION_REMOVE_ALL_METAS)
    assert(model.get_metas() == {})
"#);

    let model = UpdateUser {
        request_id: Some(123),
        auth: Arc::new(Some(UserAuth {
            user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
            session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
        })),
        target_user_id: Some(UserId::from("1ea7b016-fdd2-4d07-b71c-f877049265da".to_string())),
        name: Some("erhan".to_string()),
        email: Some("erhan@abc.com".to_string()),
        password: Some("abc".to_string()),
        device_id: Some("device_id".to_string()),
        custom_id: Some("custom_id".to_string()),
        user_type: Some(UserType::Admin),
        metas: Some(HashMap::new()),
        meta_action: MetaAction::RemoveAllMetas,
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_update_user(model).expect("pre_update_user returned Err");
    let model = executer.post_update_user(model, true).expect("post_update_user returned Err");

    
    /* Update user fields */
    let (executer, _) = create_python_environtment("user_update_test2.py", r#"
import yummy

def pre_update_user(model):
    assert(model.get_request_id() == 123)
    assert(model.get_target_user_id() == "1ea7b016-fdd2-4d07-b71c-f877049265da")
    model.set_name("baris")
    model.set_password("password")
    model.set_device_id("new_device_id")
    model.set_email("abc@abc.com")
    model.set_custom_id("new_custom_id")
    model.set_user_type(yummy.constants.USER_TYPE_MOD)
    model.set_meta_action(yummy.constants.META_ACTION_REMOVE_UNUSED_METAS)
    model.set_metas(None)

def post_update_user(model, success):
    assert(model.get_name() == "baris")
    assert(model.get_password() == "password")
    assert(model.get_device_id() == "new_device_id")
    assert(model.get_email() == "abc@abc.com")
    assert(model.get_custom_id() == "new_custom_id")
    assert(model.get_user_type() == yummy.constants.USER_TYPE_MOD)
    assert(model.get_meta_action() == yummy.constants.META_ACTION_REMOVE_UNUSED_METAS)
    assert(model.get_metas() is None)
"#);

    let model = executer.pre_update_user(model).expect("pre_update_user returned Err");
    let model = executer.post_update_user(model, true).expect("post_update_user returned Err");

    assert_eq!(model.custom_id, Some("new_custom_id".to_string()));
    assert_eq!(model.device_id, Some("new_device_id".to_string()));
    assert_eq!(model.email, Some("abc@abc.com".to_string()));
    assert_eq!(model.password, Some("password".to_string()));
    assert_eq!(model.name, Some("baris".to_string()));
    assert_eq!(model.user_type, Some(UserType::Mod));
    assert_eq!(model.meta_action, MetaAction::RemoveUnusedMetas);
    assert_eq!(model.metas, None);

    
    /* Update metas */
    let (executer, _) = create_python_environtment("user_update_test3.py", r#"
import yummy

def pre_update_user(model):
    model.set_metas({
        'meta 1': 1,
        'meta 2': True,
        'meta 3': 2.3,
        'meta 4': 'test',
        'meta 5': []
    })
"#);

    let model = executer.pre_update_user(model).expect("pre_update_user returned Err");
    let model = executer.post_update_user(model, true).expect("post_update_user returned Err");

    if let Some(metas) = model.metas.as_ref() {
        assert_eq!(metas.len(), 5);
        assert_eq!(metas.get("meta 1"), Some(&UserMetaType::Number(1.0, UserMetaAccess::User)));
        assert_eq!(metas.get("meta 2"), Some(&UserMetaType::Bool(true, UserMetaAccess::User)));
        assert_eq!(metas.get("meta 3"), Some(&UserMetaType::Number(2.3, UserMetaAccess::User)));
        assert_eq!(metas.get("meta 4"), Some(&UserMetaType::String("test".to_string(), UserMetaAccess::User)));
        assert_eq!(metas.get("meta 5"), Some(&UserMetaType::List(Box::new(Vec::new()), UserMetaAccess::User)));
    } else {
        assert!(false, "Metas information is None")
    }
    
    
    /* Get metas */
    let (executer, _) = create_python_environtment("user_update_test4.py", r#"
import yummy

def pre_update_user(model):
    assert(model.get_metas() == {
        'meta 1': 1,
        'meta 2': True,
        'meta 3': 2.3,
        'meta 4': 'test',
        'meta 5': []
    })
"#);

    let model = executer.pre_update_user(model).expect("pre_update_user returned Err");
    executer.post_update_user(model, true).expect("post_update_user returned Err");
}

#[test]
fn create_room_test() {

    let (executer, _) = create_python_environtment("create_room_test1.py", r#"
import yummy

def pre_create_room(model):
    assert(model.get_name() == "my room")
    assert(model.get_description() == "description")
    assert(model.get_max_user() == 1024)
    assert(model.get_join_request())
    assert(model.get_tags() == ["tag1", "tag2", "tag3"])
    assert(model.get_metas() == {"meta1": None, "meta2": 10.1, "meta3": None, "meta4": None})
    assert(model.get_access_type() == yummy.constants.ROOM_ACCESS_TYPE_PUBLIC)

def post_create_room(model, success):
    assert(model.get_name() == "my room")
    assert(model.get_description() == "description")
    assert(model.get_max_user() == 1024)
    assert(model.get_join_request())
    assert(model.get_tags() == ["tag1", "tag2", "tag3"])
    assert(model.get_metas() == {"meta1": None, "meta2": 10.1, "meta3": None, "meta4": None})
    assert(model.get_access_type() == yummy.constants.ROOM_ACCESS_TYPE_PUBLIC)
"#);

    let model = CreateRoomRequest {
        request_id: Some(123),
        auth: Arc::new(Some(UserAuth {
            user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
            session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
        })),
        name: Some("my room".to_string()),
        description: Some("description".to_string()),
        access_type: CreateRoomAccessType::Public,
        join_request: true,
        max_user: 1024,
        metas: Some(HashMap::from([
            ("meta1".to_string(), RoomMetaType::Null),
            ("meta2".to_string(), RoomMetaType::Number(10.1, RoomMetaAccess::User)),
            ("meta3".to_string(), RoomMetaType::Null),
            ("meta4".to_string(), RoomMetaType::Null),
        ])),
        tags: vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()],
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_create_room(model).expect("pre_create_room returned Err");
    let model = executer.post_create_room(model, true).expect("post_create_room returned Err");


    let (executer, _) = create_python_environtment("create_room_test2.py", r#"
import yummy

def pre_create_room(model):
    model.set_name("names")
    model.set_description("descriptions")
    model.set_max_user(0)
    model.set_join_request(False)
    model.set_tags(["y", "u", "m", "m", "y"])
    model.set_metas({"1": 1024})
    model.set_access_type(yummy.constants.ROOM_ACCESS_TYPE_FRIEND)

def post_create_room(model, success):
    assert(model.get_name() == "names")
    assert(model.get_description() == "descriptions")
    assert(model.get_max_user() == 0)
    assert(model.get_join_request() is False)
    assert(model.get_tags() == ["y", "u", "m", "m", "y"])
    assert(model.get_metas() == {"1": 1024})
    assert(model.get_access_type() == yummy.constants.ROOM_ACCESS_TYPE_FRIEND)
"#);

    let model = executer.pre_create_room(model).expect("pre_create_room returned Err");
    let model = executer.post_create_room(model, true).expect("post_create_room returned Err");
    
    assert_eq!(model.access_type, CreateRoomAccessType::Friend);
    assert_eq!(model.name, Some("names".to_string()));
    assert_eq!(model.description, Some("descriptions".to_string()));
    assert_eq!(model.join_request, false);
    assert_eq!(model.max_user, 0);
    assert_eq!(model.tags, vec!["y".to_string(), "u".to_string(), "m".to_string(), "m".to_string(), "y".to_string()]);

    if let Some(metas) = model.metas.as_ref() {
        assert_eq!(metas.len(), 1);
        assert_eq!(metas.get("1"), Some(&RoomMetaType::Number(1024.0, RoomMetaAccess::Anonymous)));
    } else {
        assert!(false, "Metas information is None")
    }
}

#[test]
fn update_room_test() {

    let (executer, _) = create_python_environtment("update_room_test1.py", r#"
import yummy

def pre_update_room(model):
    assert(model.get_name() is None)
    assert(model.get_description() is None)
    assert(model.get_max_user() is None)
    assert(model.get_join_request() is None)
    assert(model.get_tags() is None)
    assert(model.get_metas() is None)
    assert(model.get_access_type() is None)
    assert(model.get_meta_action() == yummy.constants.META_ACTION_ONLY_ADD_OR_UPDATE)
    assert(model.get_user_permission() is None)

def post_update_room(model, success):
    assert(model.get_name() is None)
    assert(model.get_description() is None)
    assert(model.get_max_user() is None)
    assert(model.get_join_request() is None)
    assert(model.get_tags() is None)
    assert(model.get_metas() is None)
    assert(model.get_access_type() is None)
    assert(model.get_meta_action() == yummy.constants.META_ACTION_ONLY_ADD_OR_UPDATE)
    assert(model.get_user_permission() is None)
"#);

    let model = UpdateRoom {
        request_id: Some(123),
        auth: Arc::new(Some(UserAuth {
            user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
            session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
        })),
        room_id: RoomId::new(),
        description: None,
        name: None,
        access_type: None,
        join_request: None,
        max_user: None,
        tags: None,
        metas: None,
        user_permission: None,
        meta_action: MetaAction::default(),
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_update_room(model).expect("pre_update_room returned Err");
    let model = executer.post_update_room(model, true).expect("post_update_room returned Err");


    let (executer, _) = create_python_environtment("update_room_test2.py", r#"
import yummy

def pre_update_room(model):
    model.set_name("names")
    model.set_description("descriptions")
    model.set_max_user(0)
    model.set_join_request(False)
    model.set_tags(["y", "u", "m", "m", "y"])
    model.set_metas({"1": 1024})
    model.set_access_type(yummy.constants.ROOM_ACCESS_TYPE_FRIEND)
    model.set_user_permission({
        '79df9307-7f2a-489b-9a02-ea27952462f7': yummy.constants.ROOM_USER_TYPE_OWNER,
        'faf727f1-ac60-4727-a393-1fe9387c4b5b': yummy.constants.ROOM_USER_TYPE_MODERATOR
    })

def post_update_room(model, success):
    assert(model.get_name() == "names")
    assert(model.get_description() == "descriptions")
    assert(model.get_max_user() == 0)
    assert(model.get_join_request() is False)
    assert(model.get_tags() == ["y", "u", "m", "m", "y"])
    assert(model.get_metas() == {"1": 1024})
    assert(model.get_access_type() == yummy.constants.ROOM_ACCESS_TYPE_FRIEND)
    assert(model.get_user_permission() == {
        '79df9307-7f2a-489b-9a02-ea27952462f7': yummy.constants.ROOM_USER_TYPE_OWNER,
        'faf727f1-ac60-4727-a393-1fe9387c4b5b': yummy.constants.ROOM_USER_TYPE_MODERATOR
    })
"#);

    let model = executer.pre_update_room(model).expect("pre_update_room returned Err");
    executer.post_update_room(model, true).expect("post_update_room returned Err");
}

#[test]
fn join_to_room_test() {
    let (executer, _) = create_python_environtment("join_to_room_test.py", r#"
import yummy

def pre_join_to_room(model):
    assert(model.get_room_id() == "d508b370-6249-4fd3-9b3e-3aa66577a686")
    assert(model.get_room_user_type() == yummy.constants.ROOM_USER_TYPE_USER)

    model.set_room_user_type(yummy.constants.ROOM_USER_TYPE_MODERATOR)

def post_join_to_room(model, success):
    assert(model.get_room_id() == "d508b370-6249-4fd3-9b3e-3aa66577a686")
    assert(model.get_room_user_type() == yummy.constants.ROOM_USER_TYPE_MODERATOR)
"#);

    let model = JoinToRoomRequest {
        request_id: Some(123),
        auth: Arc::new(Some(UserAuth {
            user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
            session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
        })),
        room_id: RoomId::from("d508b370-6249-4fd3-9b3e-3aa66577a686".to_string()),
        room_user_type: RoomUserType::User,
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_join_to_room(model).expect("pre_join_to_room returned Err");
    let model = executer.post_join_to_room(model, true).expect("post_join_to_room returned Err");

    assert_eq!(model.room_user_type, RoomUserType::Moderator);
}

#[test]
fn process_waiting_user_test() {
    let (executer, _) = create_python_environtment("process_waiting_user_test.py", r#"
import yummy

def pre_process_waiting_user(model):
    assert(model.get_room_id() == "d508b370-6249-4fd3-9b3e-3aa66577a686")
    assert(model.get_target_user_id() == "69531fc4-bb09-41a7-aeb0-364876f1ff79")
    assert(model.get_status() == True)

    model.set_status(False)

def post_process_waiting_user(model, success):
    assert(model.get_room_id() == "d508b370-6249-4fd3-9b3e-3aa66577a686")
    assert(model.get_target_user_id() == "69531fc4-bb09-41a7-aeb0-364876f1ff79")
    assert(model.get_status() == False)
"#);

    let model = ProcessWaitingUser {
        request_id: Some(123),
        auth: Arc::new(Some(UserAuth {
            user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
            session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
        })),
        room_id: RoomId::from("d508b370-6249-4fd3-9b3e-3aa66577a686".to_string()),
        user_id: UserId::from("69531fc4-bb09-41a7-aeb0-364876f1ff79".to_string()),
        status: true,
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_process_waiting_user(model).expect("pre_process_waiting_user returned Err");
    let model = executer.post_process_waiting_user(model, true).expect("post_process_waiting_user returned Err");

    assert_eq!(model.status, false);
}

#[test]
fn kick_user_from_room_test() {
    let (executer, _) = create_python_environtment("kick_user_from_room_test.py", r#"
import yummy

def pre_kick_user_from_room(model):
    assert(model.get_room_id() == "d508b370-6249-4fd3-9b3e-3aa66577a686")
    assert(model.get_target_user_id() == "69531fc4-bb09-41a7-aeb0-364876f1ff79")
    assert(model.get_ban() == True)

    model.set_ban(False)

def post_kick_user_from_room(model, success):
    assert(model.get_room_id() == "d508b370-6249-4fd3-9b3e-3aa66577a686")
    assert(model.get_target_user_id() == "69531fc4-bb09-41a7-aeb0-364876f1ff79")
    assert(model.get_ban() == False)
"#);

    let model = KickUserFromRoom {
        request_id: Some(123),
        auth: Arc::new(Some(UserAuth {
            user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
            session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
        })),
        room_id: RoomId::from("d508b370-6249-4fd3-9b3e-3aa66577a686".to_string()),
        user_id: UserId::from("69531fc4-bb09-41a7-aeb0-364876f1ff79".to_string()),
        ban: true,
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_kick_user_from_room(model).expect("pre_kick_user_from_room returned Err");
    let model = executer.post_kick_user_from_room(model, true).expect("post_kick_user_from_room returned Err");

    assert_eq!(model.ban, false);
}

#[test]
fn disconnect_from_room_request_test() {
    let (executer, _) = create_python_environtment("disconnect_from_room_request_test.py", r#"
import yummy

def pre_disconnect_from_room(model):
    assert(model.get_room_id() == "d508b370-6249-4fd3-9b3e-3aa66577a686")

def post_disconnect_from_room(model, success):
    assert(model.get_room_id() == "d508b370-6249-4fd3-9b3e-3aa66577a686")
"#);

    let model = DisconnectFromRoomRequest {
        request_id: Some(123),
        auth: Arc::new(Some(UserAuth {
            user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
            session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
        })),
        room_id: RoomId::from("d508b370-6249-4fd3-9b3e-3aa66577a686".to_string()),
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_disconnect_from_room(model).expect("pre_disconnect_from_room returned Err");
    executer.post_disconnect_from_room(model, true).expect("post_disconnect_from_room returned Err");
}

#[test]
fn message_to_room_test() {
    let (executer, _) = create_python_environtment("message_to_room_test.py", r#"
import yummy

def pre_message_to_room(model):
    assert(model.get_room_id() == "d508b370-6249-4fd3-9b3e-3aa66577a686")
    assert(model.get_message() == "hello")

    model.set_message("world")

def post_message_to_room(model, success):
    assert(model.get_room_id() == "d508b370-6249-4fd3-9b3e-3aa66577a686")
    assert(model.get_message() == "world")
"#);

    let model = MessageToRoomRequest {
        request_id: Some(123),
        auth: Arc::new(Some(UserAuth {
            user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
            session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
        })),
        room_id: RoomId::from("d508b370-6249-4fd3-9b3e-3aa66577a686".to_string()),
        message: serde_json::Value::String("hello".to_string()),
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_message_to_room(model).expect("pre_message_to_room returned Err");
    let model = executer.post_message_to_room(model, true).expect("post_message_to_room returned Err");
    assert_eq!(&model.message, "world");
}

#[test]
fn room_list_request_test() {
    let (executer, _) = create_python_environtment("room_list_request_test.py", r#"
import yummy

def pre_room_list_request(model):
    assert(model.get_tag() is None)
    model.set_tag("test")

    assert(model.get_members() == [yummy.constants.ROOM_INFO_TYPE_ROOM_NAME])
    model.set_members([yummy.constants.ROOM_INFO_TYPE_ROOM_NAME, yummy.constants.ROOM_INFO_TYPE_METAS])

def post_room_list_request(model, success):
    assert(model.get_tag() == "test")
    assert(model.get_members() == [yummy.constants.ROOM_INFO_TYPE_ROOM_NAME, yummy.constants.ROOM_INFO_TYPE_METAS])
"#);

    let model = RoomListRequest {
        request_id: Some(123),
        tag: None,
        members: vec![RoomInfoTypeVariant::RoomName],
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_room_list_request(model).expect("pre_room_list_request returned Err");
    let model = executer.post_room_list_request(model, true).expect("post_room_list_request returned Err");
    
    assert_eq!(&model.tag, &Some("test".to_string()));
    assert_eq!(&model.members, &vec![RoomInfoTypeVariant::RoomName, RoomInfoTypeVariant::Metas]);
}

#[test]
fn waiting_room_joins_test() {
    let (executer, _) = create_python_environtment("waiting_room_joins_test.py", r#"
import yummy

def pre_waiting_room_joins(model):
    assert(model.get_room_id() == "d508b370-6249-4fd3-9b3e-3aa66577a686")

def post_waiting_room_joins(model, success):
    assert(model.get_room_id() == "d508b370-6249-4fd3-9b3e-3aa66577a686")
"#);

    let model = WaitingRoomJoins {
        request_id: Some(123),
        auth: Arc::new(Some(UserAuth {
            user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
            session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
        })),
        room_id: RoomId::from("d508b370-6249-4fd3-9b3e-3aa66577a686".to_string()),
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_waiting_room_joins(model).expect("pre_waiting_room_joins returned Err");
    executer.post_waiting_room_joins(model, true).expect("post_waiting_room_joins returned Err");
}

#[test]
fn get_room_request_test() {
    let (executer, _) = create_python_environtment("get_room_request_test.py", r#"
import yummy

def pre_get_room_request(model):
    assert(model.get_room_id() == "d508b370-6249-4fd3-9b3e-3aa66577a686")
    assert(model.get_members() == [yummy.constants.ROOM_INFO_TYPE_ROOM_NAME])
    model.set_members([yummy.constants.ROOM_INFO_TYPE_ROOM_NAME, yummy.constants.ROOM_INFO_TYPE_METAS])

def post_get_room_request(model, success):
    assert(model.get_room_id() == "d508b370-6249-4fd3-9b3e-3aa66577a686")
    assert(model.get_members() == [yummy.constants.ROOM_INFO_TYPE_ROOM_NAME, yummy.constants.ROOM_INFO_TYPE_METAS])
"#);

    let model = GetRoomRequest {
        request_id: Some(123),
        auth: Arc::new(Some(UserAuth {
            user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
            session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
        })),
        room_id: RoomId::from("d508b370-6249-4fd3-9b3e-3aa66577a686".to_string()),
        members: vec![RoomInfoTypeVariant::RoomName],
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_get_room_request(model).expect("pre_get_room_request returned Err");
    let model = executer.post_get_room_request(model, true).expect("post_get_room_request returned Err");
    assert_eq!(&model.members, &vec![RoomInfoTypeVariant::RoomName, RoomInfoTypeVariant::Metas]);
}

/* Basic model checks */
model_tester!(device_id_auth_tester, "device_id_auth_tester.py", pre_deviceid_auth, post_deviceid_auth, DeviceIdAuthRequest {
    request_id: Some(123),
    auth: Arc::new(Some(UserAuth {
        user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
        session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
    })),
    id: "abc".to_string(),
    socket: Arc::new(DummyClient::default())
});

model_tester!(connection_user_disconnect, "connection_user_disconnect_tester.py", pre_user_disconnected, post_user_disconnected, ConnUserDisconnect {
    request_id: Some(123),
    auth: Arc::new(Some(UserAuth {
        user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
        session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
    })),
    send_message: true,
    socket: Arc::new(DummyClient::default())
});

model_tester!(email_auth_tester, "email_auth_tester.py", pre_email_auth, post_email_auth, EmailAuthRequest {
    request_id: Some(123),
    auth: Arc::new(Some(UserAuth {
        user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
        session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
    })),
    email: "abc@gmail.com".to_string(),
    password: Password::from("password123".to_string()),
    if_not_exist_create: true,
    socket: Arc::new(DummyClient::default())
});

model_tester!(custom_id_auth_tester, "custom_id_auth_tester.py", pre_customid_auth, post_customid_auth, CustomIdAuthRequest {
    request_id: Some(123),
    auth: Arc::new(Some(UserAuth {
        user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
        session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
    })),
    id: "1234567890".to_string(),
    socket: Arc::new(DummyClient::default())
});

model_tester!(logout_tester, "logout_tester.py", pre_logout, post_logout, LogoutRequest {
    request_id: Some(123),
    auth: Arc::new(Some(UserAuth {
        user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
        session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
    })),
    socket: Arc::new(DummyClient::default())
});

model_tester!(refresh_token, "refresh_token.py", pre_refresh_token, post_refresh_token, RefreshTokenRequest {
    request_id: Some(123),
    auth: Arc::new(Some(UserAuth {
        user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
        session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
    })),
    token: "TOKEN".to_string(),
    socket: Arc::new(DummyClient::default())
});

model_tester!(restore_token, "restore_token.py", pre_restore_token, post_restore_token, RestoreTokenRequest {
    request_id: Some(123),
    auth: Arc::new(Some(UserAuth {
        user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
        session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
    })),
    token: "TOKEN".to_string(),
    socket: Arc::new(DummyClient::default())
});

model_tester!(update_user, "update_user.py", pre_update_user, post_update_user, UpdateUser {
    request_id: Some(123),
    auth: Arc::new(Some(UserAuth {
        user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
        session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
    })),
    target_user_id: None,
    name: None,
    email: None,
    password: None,
    device_id: None,
    custom_id: None,
    user_type: None,
    metas: None,
    meta_action: MetaAction::default(),
    socket: Arc::new(DummyClient::default())
});

model_tester!(create_room, "create_room.py", pre_create_room, post_create_room, CreateRoomRequest {
    request_id: Some(123),
    auth: Arc::new(Some(UserAuth {
        user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
        session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
    })),
    description: None,
    name: None,
    access_type: CreateRoomAccessType::Public,
    join_request: true,
    max_user: 1024,
    tags: Vec::new(),
    metas: None,
    socket: Arc::new(DummyClient::default())
});


model_tester!(update_room, "update_room.py", pre_update_room, post_update_room, UpdateRoom {
    request_id: Some(123),
    auth: Arc::new(Some(UserAuth {
        user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
        session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
    })),
    room_id: RoomId::new(),
    description: None,
    name: None,
    access_type: None,
    join_request: None,
    max_user: None,
    tags: None,
    metas: None,
    user_permission: None,
    meta_action: MetaAction::default(),
    socket: Arc::new(DummyClient::default())
});

model_tester!(join_to_room, "join_to_room.py", pre_join_to_room, post_join_to_room, JoinToRoomRequest {
    request_id: Some(123),
    auth: Arc::new(Some(UserAuth {
        user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
        session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
    })),
    room_id: RoomId::new(),
    room_user_type: RoomUserType::default(),
    socket: Arc::new(DummyClient::default())
});

model_tester!(process_waiting_user, "process_waiting_user.py", pre_process_waiting_user, post_process_waiting_user, ProcessWaitingUser {
    request_id: Some(123),
    auth: Arc::new(Some(UserAuth {
        user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
        session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
    })),
    room_id: RoomId::new(),
    user_id: UserId::new(),
    status: true,
    socket: Arc::new(DummyClient::default())
});

model_tester!(kick_user_from_room, "kick_user_from_room.py", pre_kick_user_from_room, post_kick_user_from_room, KickUserFromRoom {
    request_id: Some(123),
    auth: Arc::new(Some(UserAuth {
        user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
        session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
    })),
    room_id: RoomId::new(),
    user_id: UserId::new(),
    ban: true,
    socket: Arc::new(DummyClient::default())
});

model_tester!(disconnect_from_room_request, "disconnect_from_room_request.py", pre_disconnect_from_room, post_disconnect_from_room, DisconnectFromRoomRequest {
    request_id: Some(123),
    auth: Arc::new(Some(UserAuth {
        user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
        session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
    })),
    room_id: RoomId::new(),
    socket: Arc::new(DummyClient::default())
});

model_tester!(message_to_room, "message_to_room.py", pre_message_to_room, post_message_to_room, MessageToRoomRequest {
    request_id: Some(123),
    auth: Arc::new(Some(UserAuth {
        user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
        session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
    })),
    room_id: RoomId::new(),
    message: serde_json::Value::String("hello".to_string()),
    socket: Arc::new(DummyClient::default())
});

model_tester!(waiting_room_joins, "waiting_room_joins.py", pre_waiting_room_joins, post_waiting_room_joins, WaitingRoomJoins {
    request_id: Some(123),
    auth: Arc::new(Some(UserAuth {
        user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
        session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
    })),
    room_id: RoomId::new(),
    socket: Arc::new(DummyClient::default())
});

model_tester!(get_room_request, "waiting_room_joins.py", pre_get_room_request, post_get_room_request, GetRoomRequest {
    request_id: Some(123),
    auth: Arc::new(Some(UserAuth {
        user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
        session: SessionId::from("1bca52a9-4b98-45dd-bda9-93468d1b583f".to_string())
    })),
    room_id: RoomId::new(),
    members: vec![RoomInfoTypeVariant::RoomName],
    socket: Arc::new(DummyClient::default())
});
