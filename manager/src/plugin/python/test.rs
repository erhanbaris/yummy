/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::{sync::Arc, env::temp_dir};
use std::io::Write;

use cache::state::{YummyState};
use general::password::Password;
use model::{UserId, SessionId};
use model::auth::UserAuth;
use model::config::YummyConfig;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use testing::cache::DummyResourceFactory;
use testing::client::DummyClient;
use testing::database::get_database_pool;

use crate::auth::model::{EmailAuthRequest, CustomIdAuthRequest, LogoutRequest};
use crate::plugin::PluginExecuter;
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