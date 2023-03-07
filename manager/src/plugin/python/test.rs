/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::{rc::Rc, cell::RefCell, sync::Arc, env::temp_dir};
use std::io::Write;

use ::model::auth::UserAuth;
use cache::state::{RoomInfoTypeVariant, YummyState};
use model::config::YummyConfig;
use model::meta::collection::UserMetaCollection;
use tempdir::TempDir;

use model::meta::{MetaType, UserMetaAccess, MetaAction, RoomMetaAccess};
use model::{UserId, UserType, CreateRoomAccessType, RoomId, RoomUserType, SessionId, UserInformationModel};
use general::password::Password;
use testing::cache::DummyResourceFactory;
use testing::client::DummyClient;
use testing::database::get_database_pool;

use crate::auth::model::ConnUserDisconnect;
use crate::conn::model::UserConnected;
use crate::plugin::PluginExecuter;
use crate::room::model::{CreateRoomRequest, UpdateRoom, JoinToRoomRequest, ProcessWaitingUser, KickUserFromRoom, DisconnectFromRoomRequest, RoomListRequest, WaitingRoomJoins, GetRoomRequest};
use crate::user::model::{GetUserInformation, GetUserInformationEnum, UpdateUser};
use crate::{plugin::{EmailAuthRequest, PluginBuilder}, auth::model::{DeviceIdAuthRequest, CustomIdAuthRequest, LogoutRequest, RefreshTokenRequest, RestoreTokenRequest}};
use super::model::DeviceIdAuthRequestWrapper;
use super::{PythonPlugin, PythonPluginInstaller, FunctionType};

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
    let (executer, state) = create_python_environtment("get_user_meta_test.py", r#"
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
    print("Merhaba {}" % model)
    return model
"#);

    let config = Arc::new(config);

    let plugin = PythonPluginInstaller::build_plugin(config);
    let model = DeviceIdAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        id: "abc".to_string(),
        socket: Arc::new(DummyClient::default())
    };

    plugin.execute(DeviceIdAuthRequestWrapper(model), "pre_deviceid_auth", FunctionType::DEVICEID_AUTH).unwrap();
}
