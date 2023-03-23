use std::collections::HashMap;
/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::{sync::Arc, env::temp_dir};
use std::io::Write;

use yummy_cache::state::{YummyState};
use yummy_general::password::Password;
use yummy_model::meta::{MetaAction, UserMetaType, UserMetaAccess};
use yummy_model::{UserId, SessionId, UserType};
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
"#);
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
    assert(yummy.get_user_meta(model.get_user_id(), "key") is None)
    
    assert(yummy.set_user_meta(model.get_user_id(), "key", "motorola"))
    assert(yummy.get_user_meta(model.get_user_id(), "key") == "motorola")

    assert(yummy.set_user_meta(model.get_user_id(), "key", 2023))
    assert(yummy.get_user_meta(model.get_user_id(), "key") == 2023)
    
    assert(yummy.set_user_meta(model.get_user_id(), "key", "motorola", 3))
    assert(yummy.get_user_meta(model.get_user_id(), "key") == "motorola")

    assert(yummy.set_user_meta(model.get_user_id(), "key", 2023, 1))
    assert(yummy.get_user_meta(model.get_user_id(), "key") == 2023)

    assert(yummy.set_user_meta(model.get_user_id(), "key", [123, True, 321.123, "test"]))
    assert(yummy.get_user_meta(model.get_user_id(), "key") == [123, True, 321.123, "test"])

    assert(yummy.set_user_meta(model.get_user_id(), "key", None))
    assert(yummy.get_user_meta(model.get_user_id(), "key") is None)

    yummy.set_user_meta(model.get_user_id(), "key1", "motorola")
    yummy.set_user_meta(model.get_user_id(), "key2", True)
    yummy.set_user_meta(model.get_user_id(), "key3", 123)
    yummy.set_user_meta(model.get_user_id(), "key4", 321.123)
    yummy.set_user_meta(model.get_user_id(), "key5", [123, True, 321.123, "test"])

    assert(yummy.get_user_metas(model.get_user_id()) == {'key': None, 'key1': 'motorola', 'key2': True, 'key3': 123.0, 'key4': 321.123, 'key5': [123.0, True, 321.123, 'test']})

    yummy.remove_user_meta(model.get_user_id(), "key1")
    assert(yummy.get_user_metas(model.get_user_id()) == {'key': None, 'key2': True, 'key3': 123.0, 'key4': 321.123, 'key5': [123.0, True, 321.123, 'test']})

    yummy.remove_user_meta(model.get_user_id(), "dummy")
    assert(yummy.get_user_metas(model.get_user_id()) == {'key': None, 'key2': True, 'key3': 123.0, 'key4': 321.123, 'key5': [123.0, True, 321.123, 'test']})

    assert(yummy.remove_user_metas(model.get_user_id()))
    assert(yummy.get_user_metas(model.get_user_id()) == {})
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
    let (executer, _) = create_python_environtment("restore_token_test.py", r#"
import yummy

def pre_restore_token(model):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "me")
    assert(model.get_user_id() is None)
    assert(model.get_val() == (None, None))

def post_restore_token(model, success):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "me")
    assert(model.get_user_id() is None)
    assert(model.get_val() == (None, None))
"#);

    let model = GetUserInformation {
        request_id: Some(123),
        query: GetUserInformationEnum::Me(Arc::new(None)),
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_get_user_information(model).expect("pre_get_user_information returned Err");
    executer.post_get_user_information(model, true).expect("post_get_user_information returned Err");


    /* Me test 2 */
    let (executer, _) = create_python_environtment("restore_token_test.py", r#"
import yummy

def pre_restore_token(model):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "me")
    assert(model.get_user_id() is None)
    assert(model.get_val() == ("294a6097-b8ea-4daa-b699-9f0c0c119c6d", None))

def post_restore_token(model, success):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "me")
    assert(model.get_user_id() is None)
    assert(model.get_val() == ("294a6097-b8ea-4daa-b699-9f0c0c119c6d", None))
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
    let (executer, _) = create_python_environtment("restore_token_test.py", r#"
import yummy

def pre_restore_token(model):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "UserViaSystem")
    assert(model.get_user_id() == "294a6097-b8ea-4daa-b699-9f0c0c119c6d")
    assert(model.get_val() == ("294a6097-b8ea-4daa-b699-9f0c0c119c6d", None))

def post_restore_token(model, success):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "UserViaSystem")
    assert(model.get_user_id() == "294a6097-b8ea-4daa-b699-9f0c0c119c6d")
    assert(model.get_val() == ("294a6097-b8ea-4daa-b699-9f0c0c119c6d", None))
"#);

    let model = GetUserInformation {
        request_id: Some(123),
        query: GetUserInformationEnum::UserViaSystem(UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string())),
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_get_user_information(model).expect("pre_get_user_information returned Err");
    executer.post_get_user_information(model, true).expect("post_get_user_information returned Err");


    /* User test 1 */
    let (executer, _) = create_python_environtment("restore_token_test.py", r#"
import yummy

def pre_restore_token(model):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "User")
    assert(model.get_user_id() == "294a6097-b8ea-4daa-b699-9f0c0c119c6d")
    assert(model.get_val() == ("294a6097-b8ea-4daa-b699-9f0c0c119c6d", None))

def post_restore_token(model, success):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "User")
    assert(model.get_user_id() == "294a6097-b8ea-4daa-b699-9f0c0c119c6d")
    assert(model.get_val() == ("294a6097-b8ea-4daa-b699-9f0c0c119c6d", None))
"#);

    let model = GetUserInformation {
        request_id: Some(123),
        query: GetUserInformationEnum::User {
            user: UserId::from("294a6097-b8ea-4daa-b699-9f0c0c119c6d".to_string()),
            requester: Arc::new(None)
        },
        socket: Arc::new(DummyClient::default())
    };

    let model = executer.pre_get_user_information(model).expect("pre_get_user_information returned Err");
    executer.post_get_user_information(model, true).expect("post_get_user_information returned Err");


    /* User test 2 */
    let (executer, _) = create_python_environtment("restore_token_test.py", r#"
import yummy

def pre_restore_token(model):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "User")
    assert(model.get_user_id() == "294a6097-b8ea-4daa-b699-9f0c0c119c6d")
    assert(model.get_val() == ("294a6097-b8ea-4daa-b699-9f0c0c119c6d", "edbe3335-3545-4d38-a2f6-3856e63bfd6f"))

def post_restore_token(model, success):
    assert(model.get_request_id() == 123)
    assert(model.get_query_type() == "User")
    assert(model.get_user_id() == "294a6097-b8ea-4daa-b699-9f0c0c119c6d")
    assert(model.get_val() == ("294a6097-b8ea-4daa-b699-9f0c0c119c6d", "edbe3335-3545-4d38-a2f6-3856e63bfd6f"))
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
    assert(model.get_device_id() == "device_id")
    assert(model.get_custom_id() == "custom_id")
    assert(model.get_user_type() == yummy.USER_TYPE_ADMIN)
    assert(model.get_meta_action() == yummy.META_ACTION_REMOVE_ALL_METAS)
    assert(model.get_metas() == {})

def post_update_user(model, success):
    assert(model.get_request_id() == 123)
    assert(model.get_target_user_id() == "1ea7b016-fdd2-4d07-b71c-f877049265da")
    assert(model.get_name() == "erhan")
    assert(model.get_password() == "abc")
    assert(model.get_device_id() == "device_id")
    assert(model.get_custom_id() == "custom_id")
    assert(model.get_user_type() == yummy.USER_TYPE_ADMIN)
    assert(model.get_meta_action() == yummy.META_ACTION_REMOVE_ALL_METAS)
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
        meta_action: Some(MetaAction::RemoveAllMetas),
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
    model.set_custom_id("new_custom_id")
    model.set_user_type(yummy.USER_TYPE_MOD)
    model.set_meta_action(yummy.META_ACTION_REMOVE_UNUSED_METAS)
    model.set_metas(None)

def post_update_user(model, success):
    assert(model.get_name() == "baris")
    assert(model.get_password() == "password")
    assert(model.get_device_id() == "new_device_id")
    assert(model.get_custom_id() == "new_custom_id")
    assert(model.get_user_type() == yummy.USER_TYPE_MOD)
    assert(model.get_meta_action() == yummy.META_ACTION_REMOVE_UNUSED_METAS)
    assert(model.get_metas() is None)
"#);

    let model = executer.pre_update_user(model).expect("pre_update_user returned Err");
    let model = executer.post_update_user(model, true).expect("post_update_user returned Err");

    assert_eq!(model.custom_id, Some("new_custom_id".to_string()));
    assert_eq!(model.device_id, Some("new_device_id".to_string()));
    assert_eq!(model.password, Some("password".to_string()));
    assert_eq!(model.name, Some("baris".to_string()));
    assert_eq!(model.user_type, Some(UserType::Mod));
    assert_eq!(model.meta_action, Some(MetaAction::RemoveUnusedMetas));
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
    meta_action: None,
    socket: Arc::new(DummyClient::default())
});
