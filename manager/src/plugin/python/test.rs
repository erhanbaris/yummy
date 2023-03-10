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

use testing::cache::DummyResourceFactory;
use testing::client::DummyClient;
use testing::database::get_database_pool;

use crate::plugin::{PluginExecuter, YummyPlugin};
use crate::{plugin::{PluginBuilder}, auth::model::{DeviceIdAuthRequest}};
use super::model::DeviceIdAuthRequestWrapper;
use super::{PythonPluginInstaller, FunctionType};

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
    config.python_files_path = temp_dir().to_str().unwrap().to_string();

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
fn get_user_meta_test() {
    create_python_environtment("get_user_meta_test.py", r#"
def test(model: dict):
    print("Merhaba dünya")
    return 123
"#);
}

#[test]
fn test_1() {
    let mut config = YummyConfig::default();
    create_python_file("get_user_meta_test.py", &mut config, r#"
def pre_deviceid_auth(model):
    assert(model.get_device_id() == "abc")
    model.set_device_id("erhan")
    assert(model.get_device_id() == "erhan")

    assert(model.get_request_id() == 123)
    model.set_request_id(None)
    assert(model.get_request_id() is None)

def post_deviceid_auth(model, success):
    assert(success)
    assert(model.get_device_id() == "abc")
    model.set_device_id("erhan")
    assert(model.get_device_id() == "erhan")

    assert(model.get_request_id() == 123)
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
    plugin.pre_deviceid_auth(model.clone()).unwrap();
    plugin.post_deviceid_auth(model, true).unwrap();
}
