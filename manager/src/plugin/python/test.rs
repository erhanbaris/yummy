/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* **************************************************************************************************************** */

use std::cell::RefCell;
use std::rc::Rc;
/* **************************************************************************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::{sync::Arc, env::temp_dir};
use std::io::Write;


use cache::state::{YummyState};
use model::config::YummyConfig;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use testing::cache::DummyResourceFactory;
use testing::client::DummyClient;
use testing::database::get_database_pool;

use crate::plugin::{PluginExecuter, YummyPlugin};
use crate::{plugin::{PluginBuilder}, auth::model::{DeviceIdAuthRequest}};
use super::PythonPluginInstaller;

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
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
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
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
    let mut config = YummyConfig::default();
    create_python_file("deviceid_auth_test.py", &mut config, r#"
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

    assert(model.get_request_id() is None)
    model.set_request_id(None)
    assert(model.get_request_id() is None)
"#);

    let config = Arc::new(config);

    let plugin = PythonPluginInstaller::build_plugin(config);
    let model = DeviceIdAuthRequest {
        request_id: Some(123),
        auth: Arc::new(None),
        id: "abc".to_string(),
        socket: Arc::new(DummyClient::default())
    };

    let model = Rc::new(RefCell::new(model));
    plugin.pre_deviceid_auth(model.clone()).expect("pre_deviceid_auth returned Err");
    plugin.post_deviceid_auth(model, true).expect("post_deviceid_auth returned Err");
}

#[test]
fn validation_exception_test() {
    let mut config = YummyConfig::default();
    create_python_file("validation_exception_test.py", &mut config, r#"
import yummy
def pre_deviceid_auth(model):
    yummy.fail("fail")
"#);

    let config = Arc::new(config);

    let plugin = PythonPluginInstaller::build_plugin(config);
    let model = DeviceIdAuthRequest {
        request_id: Some(123),
        auth: Arc::new(None),
        id: "abc".to_string(),
        socket: Arc::new(DummyClient::default())
    };

    let model = Rc::new(RefCell::new(model));
    let result = plugin.pre_deviceid_auth(model.clone());
    if let Err(error) = result {
        println!("{}", error.to_string());
        assert!(error.to_string() == "fail".to_string());
    } else {
        assert!(false, "No python raise received")
    }
}
