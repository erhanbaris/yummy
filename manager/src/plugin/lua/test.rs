use std::{rc::Rc, cell::RefCell, sync::Arc, env::temp_dir};
use std::io::Write;

use general::auth::UserAuth;
use general::state::RoomInfoTypeVariant;
use tempdir::TempDir;

use general::meta::{MetaType, UserMetaAccess, MetaAction, RoomMetaAccess};
use general::model::{UserId, UserType, CreateRoomAccessType, RoomId, RoomUserType, SessionId};
use general::{password::Password, config::YummyConfig};

use crate::auth::model::ConnUserDisconnect;
use crate::conn::model::UserConnected;
use crate::room::model::{CreateRoomRequest, UpdateRoom, JoinToRoomRequest, ProcessWaitingUser, KickUserFromRoom, DisconnectFromRoomRequest, RoomListRequest, WaitingRoomJoins, GetRoomRequest};
use crate::user::model::{GetUserInformation, GetUserInformationEnum, UpdateUser};
use crate::{plugin::{EmailAuthRequest, PluginBuilder, lua::LuaPluginInstaller}, auth::model::{DeviceIdAuthRequest, CustomIdAuthRequest, LogoutRequest, RefreshTokenRequest, RestoreTokenRequest}};
use super::LuaPlugin;

#[test]
fn executest_1() -> anyhow::Result<()> {
    let plugin = LuaPlugin::new();
    plugin.execute(Rc::new(RefCell::new(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        email: "".to_string(),
        password: Password::from("123456".to_string()),
        if_not_exist_create: false,
        socket: Arc::new(general::test::DummyClient::default())
    } )), "pre_email_auth")?;
    Ok(())
}

#[test]
fn lua_code_empty() {
    let mut plugin = LuaPlugin::new();
    plugin.set_content("").unwrap();
}

#[test]
#[should_panic]
fn lua_code_has_problem() {
    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"
j = 10 123
"#).unwrap();
}

#[test]
fn no_lua_function() {
    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"
j = 123
"#).unwrap();
}

#[test]
fn valid_lua_function() {
    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"
function pre_email_auth(model)
end
"#).unwrap();

plugin.execute(Rc::new(RefCell::new(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        email: "".to_string(),
        password: Password::from("123456".to_string()),
        if_not_exist_create: false,
        socket: Arc::new(general::test::DummyClient::default())
    } )), "pre_email_auth").unwrap();
}

#[test]
fn change_email_address() {
    let model = Rc::new(RefCell::new(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        email: "".to_string(),
        password: Password::from("123456".to_string()),
        if_not_exist_create: false,
        socket: Arc::new(general::test::DummyClient::default())
    } ));

    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"
    function pre_email_auth(model)
        model:get_user_id()
        model:get_session_id()
        model:get_password()
        model:get_create()

        model:set_email("erhan@erhan.com")
        model:set_password("SECRET")
        model:set_create(true)
    end
    "#).unwrap();

    plugin.execute(model.clone(), "pre_email_auth").unwrap();
    assert_eq!(&model.borrow().email, "erhan@erhan.com");
    assert_eq!(model.borrow().password.get(), "SECRET");
    assert_eq!(model.borrow().if_not_exist_create, true);
}

#[test]
fn execution_result() {
    let model = Rc::new(RefCell::new(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        email: "".to_string(),
        password: Password::from("123456".to_string()),
        if_not_exist_create: false,
        socket: Arc::new(general::test::DummyClient::default())
    } ));

    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"
    function post_email_auth(model, successed)
        model:get_user_id()
        model:get_session_id()
        model:get_password()
        model:get_create()

        model:set_email("erhan@erhan.com")
        model:set_password("SECRET")
        model:set_create(true)
    end
    "#).unwrap();

    plugin.execute_with_result(model.clone(), true, "post_email_auth").unwrap();
    assert_eq!(&model.borrow().email, "erhan@erhan.com");
    assert_eq!(model.borrow().password.get(), "SECRET");
    assert_eq!(model.borrow().if_not_exist_create, true);
}

#[test]
fn fail_test() {
    let model = Rc::new(RefCell::new(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        email: "old@email.com".to_string(),
        password: Password::from("123456".to_string()),
        if_not_exist_create: false,
        socket: Arc::new(general::test::DummyClient::default())
    } ));

    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"
    function post_email_auth(model, successed)
        error("Problem problem problem")

        model:set_email("erhan@erhan.com")
        model:set_password("SECRET")
        model:set_create(true)
    end
    "#).unwrap();

    plugin.execute_with_result(model.clone(), true, "post_email_auth").unwrap_err();
    assert_eq!(&model.borrow().email, "old@email.com");
    assert_eq!(model.borrow().password.get(), "123456");
    assert_eq!(model.borrow().if_not_exist_create, false);
}

#[test]
fn string_upper() {
    let model = Rc::new(RefCell::new(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        email: "small@email.com".to_string(),
        password: Password::from("123456".to_string()),
        if_not_exist_create: false,
        socket: Arc::new(general::test::DummyClient::default())
    } ));

    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"
    function pre_email_auth(model)
        local email = model:get_email()
        model:set_email(email:upper())
    end
    "#).unwrap();

    plugin.execute(model.clone(), "pre_email_auth").unwrap();
    assert_eq!(&model.borrow().email, "SMALL@EMAIL.COM");
}

#[test]
fn multi_function() {
    let model = Rc::new(RefCell::new(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        email: "small@email.com".to_string(),
        password: Password::from("123456".to_string()),
        if_not_exist_create: false,
        socket: Arc::new(general::test::DummyClient::default())
    } ));

    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"
    function pre_email_auth(model)
    end

    function post_email_auth(model, successed)
    end
    "#).unwrap();

    plugin.execute(model.clone(), "pre_email_auth").unwrap();
    plugin.execute_with_result(model.clone(), true, "post_email_auth").unwrap();
}

#[test]
fn save_to_table() {
    let model = Rc::new(RefCell::new(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        email: "small@email.com".to_string(),
        password: Password::from("123456".to_string()),
        if_not_exist_create: false,
        socket: Arc::new(general::test::DummyClient::default())
    } ));

    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"
    messages = {}

    function pre_email_auth(model)
        messages.pre_auth = model
    end

    function post_email_auth(model, successed)
        messages.post_auth = model
    end
    "#).unwrap();

    plugin.execute(model.clone(), "pre_email_auth").unwrap();
    plugin.execute_with_result(model.clone(), true, "post_email_auth").unwrap();

    assert!(Rc::strong_count(&model) <= 3); // This is not good, but, I dont have a solution yet
}


#[test]
fn lua_assert_check() {
    let model = Rc::new(RefCell::new(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        email: "small@email.com".to_string(),
        password: Password::from("123456".to_string()),
        if_not_exist_create: false,
        socket: Arc::new(general::test::DummyClient::default())
    } ));

    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"

    function pre_email_auth(model)
        assert(false)
    end
    "#).unwrap();

    plugin.execute(model.clone(), "pre_email_auth").unwrap_err();
}

#[test]
fn device_id_checks() {
    let model = Rc::new(RefCell::new(DeviceIdAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        id: "abc".to_string(),
        socket: Arc::new(general::test::DummyClient::default())
    } ));

    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"

    function pre_deviceid_auth(model)
        model:get_user_id()
        model:get_session_id()
        model:get_id()

        model:set_id("123")
    end

    function post_deviceid_auth(model)
        model:get_user_id()
        model:get_session_id()
        model:get_id()

        model:set_id("123")
    end
    "#).unwrap();

    plugin.execute(model.clone(), "pre_deviceid_auth").unwrap();
    plugin.execute_with_result(model.clone(), true, "post_deviceid_auth").unwrap();

    assert_eq!(&model.borrow().id, "123");
}

#[test]
fn custom_id_checks() {
    let model = Rc::new(RefCell::new(CustomIdAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        id: "abc".to_string(),
        socket: Arc::new(general::test::DummyClient::default())
    } ));

    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"

    function pre_customid_auth(model)
        model:get_user_id()
        model:get_session_id()
        model:get_id()

        model:set_id("123")
    end

    function post_customid_auth(model)
        model:get_user_id()
        model:get_session_id()
        model:get_id()

        model:set_id("123")
    end
    "#).unwrap();

    plugin.execute(model.clone(), "pre_customid_auth").unwrap();
    plugin.execute_with_result(model.clone(), true, "post_customid_auth").unwrap();

    assert_eq!(&model.borrow().id, "123");
}


#[test]
fn logout_checks() {
    let model = Rc::new(RefCell::new(LogoutRequest {
        request_id: None,
        auth: Arc::new(None),
        socket: Arc::new(general::test::DummyClient::default())
    } ));

    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"

    function pre_logout(model)
        model:get_user_id()
        model:get_session_id()
    end

    function post_logout(model)
        model:get_user_id()
        model:get_session_id()
    end
    "#).unwrap();

    plugin.execute(model.clone(), "pre_logout").unwrap();
    plugin.execute_with_result(model.clone(), true, "post_logout").unwrap();
}


#[test]
fn refresh_token_checks() {
    let model = Rc::new(RefCell::new(RefreshTokenRequest {
        request_id: None,
        auth: Arc::new(None),
        token: "token".to_string(),
        socket: Arc::new(general::test::DummyClient::default())
    } ));

    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"

    function pre_refresh_token(model)
        model:get_user_id()
        model:get_session_id()
        model:get_token()

        model:set_token("new token")
    end

    function post_refresh_token(model)
        model:get_user_id()
        model:get_session_id()
        model:get_token()
    end
    "#).unwrap();

    plugin.execute(model.clone(), "pre_refresh_token").unwrap();
    plugin.execute_with_result(model.clone(), true, "post_refresh_token").unwrap();

    assert_eq!(&model.borrow().token, "new token");
}


#[test]
fn restore_token_checks() {
    let model = Rc::new(RefCell::new(RestoreTokenRequest {
        request_id: None,
        auth: Arc::new(None),
        token: "token".to_string(),
        socket: Arc::new(general::test::DummyClient::default())
    } ));

    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"

    function pre_restore_token(model)
        model:get_user_id()
        model:get_session_id()
        model:get_token()

        model:set_token("new token")
    end

    function post_restore_token(model)
        model:get_user_id()
        model:get_session_id()
        model:get_token()
    end
    "#).unwrap();

    plugin.execute(model.clone(), "pre_restore_token").unwrap();
    plugin.execute_with_result(model.clone(), true, "post_restore_token").unwrap();

    assert_eq!(&model.borrow().token, "new token");
}


#[test]
fn user_connected_checks() {
    let model = Rc::new(RefCell::new(UserConnected {
        user_id: Arc::new(UserId::new()),
        socket: Arc::new(general::test::DummyClient::default())
    } ));

    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"

    function pre_user_connected(model)
        model:get_user_id()
    end

    function post_user_connected(model)
        model:get_user_id()
    end
    "#).unwrap();

    plugin.execute(model.clone(), "pre_user_connected").unwrap();
    plugin.execute_with_result(model.clone(), true, "post_user_connected").unwrap();
}


#[test]
fn user_disconnected_checks() {
    let model = Rc::new(RefCell::new(ConnUserDisconnect {
        request_id: None,
        auth: Arc::new(None),
        send_message: true,
        socket: Arc::new(general::test::DummyClient::default())
    } ));

    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"

    function pre_user_disconnected(model)
        model:get_user_id()
        model:get_session_id()
        model:get_send_message()

        model:set_send_message(false)
    end

    function post_user_disconnected(model)
        model:get_user_id()
        model:get_session_id()
        model:get_send_message()
    end
    "#).unwrap();

    plugin.execute(model.clone(), "pre_user_disconnected").unwrap();
    plugin.execute_with_result(model.clone(), true, "post_user_disconnected").unwrap();

    assert_eq!(model.borrow().send_message, false);
}

#[test]
fn get_user_informations_checks() {

    // Me 
    let model = Rc::new(RefCell::new(GetUserInformation {
        request_id: None,
        query: GetUserInformationEnum::Me(Arc::new(None)),
        socket: Arc::new(general::test::DummyClient::default())
    } ));

    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"
    function pre_get_user_information(model)
        query = model:get_query()
        table = query:as_table()

        assert(query:get_type(), "Me")
    end

    function post_get_user_information(model)
    end
    "#).unwrap();

    plugin.execute(model.clone(), "pre_get_user_information").unwrap();
    plugin.execute_with_result(model.clone(), true, "post_get_user_information").unwrap();


    // UserViaSystem 
    let model = Rc::new(RefCell::new(GetUserInformation {
        request_id: None,
        query: GetUserInformationEnum::UserViaSystem(UserId::new()),
        socket: Arc::new(general::test::DummyClient::default())
    } ));

    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"
    function pre_get_user_information(model)
        query = model:get_query()
        table = query:as_table()

        assert(query:get_type(), "UserViaSystem")
    end

    function post_get_user_information(model)
    end
    "#).unwrap();

    plugin.execute(model.clone(), "pre_get_user_information").unwrap();
    plugin.execute_with_result(model.clone(), true, "post_get_user_information").unwrap();


    // User 
    let model = Rc::new(RefCell::new(GetUserInformation {
        request_id: None,
        query: GetUserInformationEnum::User { user: UserId::new(), requester: Arc::new(None) },
        socket: Arc::new(general::test::DummyClient::default())
    } ));

    let mut plugin = LuaPlugin::new();
    plugin.set_content(r#"
    function pre_get_user_information(model)
        query = model:get_query()
        table = query:as_table()

        assert(query:get_type(), "User")
    end

    function post_get_user_information(model)
    end
    "#).unwrap();

    plugin.execute(model.clone(), "pre_get_user_information").unwrap();
    plugin.execute_with_result(model.clone(), true, "post_get_user_information").unwrap();

}

#[test]
fn plugin_builder_test() {
    let mut config = YummyConfig::default();
    config.lua_files_path = temp_dir().to_str().unwrap().to_string();
    
    let path = std::path::Path::new(&config.lua_files_path[..]).join("test.lua").to_string_lossy().to_string();
    let mut lua_file = std::fs::File::create(path).expect("create failed");
    lua_file.write_all(r#"
function pre_email_auth(model)
    model:set_email("erhan@erhan.com")
end

function post_email_auth(model, successed)
    assert(model:get_email() == "erhan@erhan.com")
end
"#.as_bytes()).expect("write failed");

    let model = EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        email: "".to_string(),
        password: Password::from("123456".to_string()),
        if_not_exist_create: false,
        socket: Arc::new(general::test::DummyClient::default())
    };

    let config = Arc::new(config);
    let mut builder = PluginBuilder::default();
    builder.add_installer(Box::new(LuaPluginInstaller::default()));

    let executer = Arc::new(builder.build(config));

    let model = executer.pre_email_auth(model).unwrap();
    let model = executer.post_email_auth(model, true).unwrap();
    
    assert_eq!(&model.email, "erhan@erhan.com");

}

#[test]
fn create_meta_test() {
    let mut config = YummyConfig::default();
    
    let dir = TempDir::new("test").unwrap();
    let file_path = dir.path().join("create_meta_test.lua");

    config.lua_files_path = dir.path().to_str().unwrap().to_string();
    
    let path = file_path.to_string_lossy().to_string();
    let mut lua_file = std::fs::File::create(path).expect("create failed");
    lua_file.write_all(r#"
function do_tables_match( a, b )
    return table.concat(a) == table.concat(b)
end

function pre_update_user(model)
    metas = {}
    metas.string_value = new_user_meta("Test", 6)
    metas.number_value = new_user_meta(123456, 5)
    metas.bool_value = new_user_meta(true, 4)
    array = {}
    array[1] = true
    array[2] = "2"
    array[3] = 3
    array[4] = {}
    metas.array_value = new_user_meta(array, 3)

    model:set_metas(metas)
end

function post_update_user(model, successed)
    metas = model:get_metas()
    assert(metas.string_value:get_value() == "Test")
    assert(metas.number_value:get_value() == 123456)
    assert(metas.bool_value:get_value() == true)

    array = metas.array_value:get_value()
    assert(array ~= nil)

    assert(array[1] == true)
    assert(array[2] == "2")
    assert(array[3] == 3)
    assert(do_tables_match(array[4], {}))

    assert(metas.string_value:get_access_level() == 6)
    assert(metas.number_value:get_access_level() == 5)
    assert(metas.bool_value:get_access_level() == 4)
    assert(metas.array_value:get_access_level() == 3)

    assert(metas.string_value:get_type() == 2)
    assert(metas.number_value:get_type() == 1)
    assert(metas.bool_value:get_type() == 3)
    assert(metas.array_value:get_type() == 4)
end
"#.as_bytes()).expect("write failed");

    let model = UpdateUser {
        request_id: None,
        auth: Arc::new(None),
        email: None,
        password: None,
        socket: Arc::new(general::test::DummyClient::default()),
        target_user_id: None,
        name: None,
        device_id: None,
        custom_id: None,
        user_type: None,
        metas: None,
        meta_action: None,
    };

    let config = Arc::new(config);
    let mut builder = PluginBuilder::default();
    builder.add_installer(Box::new(LuaPluginInstaller::default()));

    let executer = Arc::new(builder.build(config));

    let model = executer.pre_update_user(model).unwrap();
    let model = executer.post_update_user(model, true).unwrap();
    
    let metas = model.metas.unwrap();
    assert_eq!(metas.len(), 4);
    assert_eq!(metas.get("string_value").unwrap(), &MetaType::String("Test".to_string(), UserMetaAccess::System));
    assert_eq!(metas.get("number_value").unwrap(), &MetaType::Number(123456.0, UserMetaAccess::Admin));
    assert_eq!(metas.get("bool_value").unwrap(), &MetaType::Bool(true, UserMetaAccess::Mod));
    assert_eq!(metas.get("array_value").unwrap(), &MetaType::List(Box::new(vec![MetaType::Bool(true, UserMetaAccess::Anonymous), MetaType::String("2".to_string(), UserMetaAccess::Anonymous), MetaType::Number(3.0, UserMetaAccess::Anonymous), MetaType::List(Box::new(Vec::new()), UserMetaAccess::Anonymous)]), UserMetaAccess::Me));
}


#[test]
fn user_update_test() {
    let mut config = YummyConfig::default();
    config.lua_files_path = temp_dir().to_str().unwrap().to_string();
    
    let path = std::path::Path::new(&config.lua_files_path[..]).join("user_update_test.lua").to_string_lossy().to_string();
    let mut lua_file = std::fs::File::create(path).expect("create failed");
    lua_file.write_all(r#"
function pre_update_user(model)
    model:set_name("erhan")
    model:set_email("erhan@erhan.com")
    model:set_password("123456")
    model:set_device_id("234567890")
    model:set_custom_id("0987654321")
    model:set_user_type(1)
    model:set_meta_action(2)
    metas = {}
    metas.string_value = new_user_meta("Test", 6)
    model:set_metas(metas)
end

function post_update_user(model, successed)
    assert(model:get_email() == "erhan@erhan.com")
    assert(model:get_name() == "erhan")
    assert(model:get_password() == "123456")
    assert(model:get_device_id() == "234567890")
    assert(model:get_custom_id() == "0987654321")
    assert(model:get_user_type() == 1)
    assert(model:get_meta_action() == 2)
end
"#.as_bytes()).expect("write failed");

    let model = UpdateUser {
        request_id: None,
        auth: Arc::new(None),
        email: None,
        password: None,
        socket: Arc::new(general::test::DummyClient::default()),
        target_user_id: None,
        name: None,
        device_id: None,
        custom_id: None,
        user_type: None,
        metas: None,
        meta_action: None,
    };

    let config = Arc::new(config);
    let mut builder = PluginBuilder::default();
    builder.add_installer(Box::new(LuaPluginInstaller::default()));

    let executer = Arc::new(builder.build(config));

    let model = executer.pre_update_user(model).unwrap();
    let model = executer.post_update_user(model, true).unwrap();
    
    assert_eq!(model.email, Some("erhan@erhan.com".to_string()));
    assert_eq!(model.name, Some("erhan".to_string()));
    assert_eq!(model.password, Some("123456".to_string()));
    assert_eq!(model.device_id, Some("234567890".to_string()));
    assert_eq!(model.custom_id, Some("0987654321".to_string()));
    assert_eq!(model.user_type, Some(UserType::User));
    assert_eq!(model.meta_action, Some(MetaAction::RemoveAllMetas));

    let metas = model.metas.unwrap();
    assert_eq!(metas.len(), 1);
    assert_eq!(metas.get("string_value").unwrap(), &MetaType::String("Test".to_string(), UserMetaAccess::System));
}

#[test]
fn create_room_test() {
    let mut config = YummyConfig::default();
    config.lua_files_path = temp_dir().to_str().unwrap().to_string();
    
    let path = std::path::Path::new(&config.lua_files_path[..]).join("create_room_test.lua").to_string_lossy().to_string();
    let mut lua_file = std::fs::File::create(path).expect("create failed");
    lua_file.write_all(r#"
function pre_create_room(model)
    model:set_name("erhan")
    model:set_description("desc")
    model:set_access_type(2)
    model:set_join_request(true)
    model:set_max_user(10)

    array = {}
    array[1] = "1"
    array[2] = "2"
    array[3] = "3"
    array[4] = "4"
    
    model:set_tags(array)

    metas = {}
    metas.string_value = new_room_meta("Test", 6)
    model:set_metas(metas)
end

function post_create_room(model, successed)
    assert(model:get_name() == "erhan")
    assert(model:get_description() == "desc")
    assert(model:get_access_type() == 2)
    assert(model:get_join_request() == true)
    assert(model:get_max_user() == 10)
end
"#.as_bytes()).expect("write failed");

    let model = CreateRoomRequest {
        request_id: None,
        auth: Arc::new(None),
        socket: Arc::new(general::test::DummyClient::default()),
        name: None,
        metas: None,
        description: None,
        access_type: CreateRoomAccessType::Public,
        join_request: false,
        max_user: 0,
        tags: Vec::new(),
    };

    let config = Arc::new(config);
    let mut builder = PluginBuilder::default();
    builder.add_installer(Box::new(LuaPluginInstaller::default()));

    let executer = Arc::new(builder.build(config));

    let model = executer.pre_create_room(model).unwrap();
    let model = executer.post_create_room(model, true).unwrap();

    assert_eq!(model.name, Some("erhan".to_string()));
    assert_eq!(model.description, Some("desc".to_string()));
    assert_eq!(model.access_type, CreateRoomAccessType::Friend);
    assert_eq!(model.join_request, true);
    assert_eq!(model.max_user, 10);
    assert_eq!(model.tags, vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()]);

    let metas = model.metas.unwrap();
    assert_eq!(metas.len(), 1);
    assert_eq!(metas.get("string_value").unwrap(), &MetaType::String("Test".to_string(), RoomMetaAccess::Anonymous));
}

#[test]
fn update_room_test() {
    let mut config = YummyConfig::default();
    config.lua_files_path = temp_dir().to_str().unwrap().to_string();
    
    let path = std::path::Path::new(&config.lua_files_path[..]).join("update_room_test.lua").to_string_lossy().to_string();
    let mut lua_file = std::fs::File::create(path).expect("create failed");
    lua_file.write_all(r#"
function pre_update_room(model)
    model:set_name("erhan")
    model:set_description("desc")
    model:set_access_type(2)
    model:set_join_request(true)
    model:set_max_user(10)

    array = {}
    array[1] = "1"
    array[2] = "2"
    array[3] = "3"
    array[4] = "4"
    
    model:set_tags(array)

    metas = {}
    metas.string_value = new_room_meta("Test", 6)
    model:set_metas(metas)

    permissions = {}
    permissions["0fea69b6-4032-4ea4-9a32-72e818ce11da"] = 1
    permissions["0fea69b6-4032-4ea4-9a32-72e818ce11db"] = 2
    permissions["0fea69b6-4032-4ea4-9a32-72e818ce11dc"] = 3
    model:set_user_permission(permissions)
end

function post_update_room(model, successed)
    assert(model:get_name() == "erhan")
    assert(model:get_description() == "desc")
    assert(model:get_access_type() == 2)
    assert(model:get_join_request() == true)
    assert(model:get_max_user() == 10)
end
"#.as_bytes()).expect("write failed");

    let model = UpdateRoom {
        request_id: None,
        auth: Arc::new(None),
        socket: Arc::new(general::test::DummyClient::default()),
        name: None,
        metas: None,
        description: None,
        access_type: None,
        join_request: None,
        max_user: None,
        tags: None,
        room_id: RoomId::new(),
        meta_action: None,
        user_permission: None,
    };

    let config = Arc::new(config);
    let mut builder = PluginBuilder::default();
    builder.add_installer(Box::new(LuaPluginInstaller::default()));

    let executer = Arc::new(builder.build(config));

    let model = executer.pre_update_room(model).unwrap();
    let model = executer.post_update_room(model, true).unwrap();

    assert_eq!(model.name, Some("erhan".to_string()));
    assert_eq!(model.description, Some("desc".to_string()));
    assert_eq!(model.access_type, Some(CreateRoomAccessType::Friend));
    assert_eq!(model.join_request, Some(true));
    assert_eq!(model.max_user, Some(10));
    assert_eq!(model.tags, Some(vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()]));

    let metas = model.metas.unwrap();
    assert_eq!(metas.len(), 1);
    assert_eq!(metas.get("string_value").unwrap(), &MetaType::String("Test".to_string(), RoomMetaAccess::Anonymous));

    let user_permission = model.user_permission.unwrap();
    assert_eq!(user_permission.len(), 3);
    assert_eq!(user_permission.get(&UserId::from("0fea69b6-4032-4ea4-9a32-72e818ce11da".to_string())).unwrap(), &RoomUserType::User);
    assert_eq!(user_permission.get(&UserId::from("0fea69b6-4032-4ea4-9a32-72e818ce11db".to_string())).unwrap(), &RoomUserType::Moderator);
    assert_eq!(user_permission.get(&UserId::from("0fea69b6-4032-4ea4-9a32-72e818ce11dc".to_string())).unwrap(), &RoomUserType::Owner);
}

#[test]
fn join_to_room_test() {
    let mut config = YummyConfig::default();
    config.lua_files_path = temp_dir().to_str().unwrap().to_string();
    
    let path = std::path::Path::new(&config.lua_files_path[..]).join("join_to_room_test.lua").to_string_lossy().to_string();
    let mut lua_file = std::fs::File::create(path).expect("create failed");
    lua_file.write_all(r#"
function pre_join_to_room(model)
    model:set_room("0fea69b6-4032-4ea4-9a32-72e818ce11da")
    model:set_room_user_type(3)
end

function post_join_to_room(model, successed)
    print(model:get_room())
    print(model:get_room_user_type())
    assert(model:get_room() == "0fea69b6-4032-4ea4-9a32-72e818ce11da")
    assert(model:get_room_user_type() == 3)
end
"#.as_bytes()).expect("write failed");

    let model = JoinToRoomRequest {
        request_id: None,
        auth: Arc::new(None),
        socket: Arc::new(general::test::DummyClient::default()),
        room: RoomId::new(),
        room_user_type: RoomUserType::User
    };

    let config = Arc::new(config);
    let mut builder = PluginBuilder::default();
    builder.add_installer(Box::new(LuaPluginInstaller::default()));

    let executer = Arc::new(builder.build(config));

    let model = executer.pre_join_to_room(model).unwrap();
    let model = executer.post_join_to_room(model, true).unwrap();

    assert_eq!(model.room, RoomId::from("0fea69b6-4032-4ea4-9a32-72e818ce11da".to_string()));
    assert_eq!(model.room_user_type, RoomUserType::Owner);
}

#[test]
fn process_waiting_user_test() {
    let mut config = YummyConfig::default();
    config.lua_files_path = temp_dir().to_str().unwrap().to_string();
    
    let path = std::path::Path::new(&config.lua_files_path[..]).join("process_waiting_user_test.lua").to_string_lossy().to_string();
    let mut lua_file = std::fs::File::create(path).expect("create failed");
    lua_file.write_all(r#"
function pre_process_waiting_user(model)
    model:set_room("0fea69b6-4032-4ea4-9a32-72e818ce11da")
    model:set_user("0fea69b6-4032-4ea4-9a32-72e818ce11db")
    model:set_status(true)
end

function post_process_waiting_user(model, successed)
    assert(model:get_room() == "0fea69b6-4032-4ea4-9a32-72e818ce11da")
    assert(model:get_user() == "0fea69b6-4032-4ea4-9a32-72e818ce11db")
    assert(model:get_status() == true)
end
"#.as_bytes()).expect("write failed");

    let model = ProcessWaitingUser {
        request_id: None,
        auth: Arc::new(None),
        socket: Arc::new(general::test::DummyClient::default()),
        room: RoomId::new(),
        user: UserId::new(),
        status: false
    };

    let config = Arc::new(config);
    let mut builder = PluginBuilder::default();
    builder.add_installer(Box::new(LuaPluginInstaller::default()));

    let executer = Arc::new(builder.build(config));

    let model = executer.pre_process_waiting_user(model).unwrap();
    let model = executer.post_process_waiting_user(model, true).unwrap();

    assert_eq!(model.room, RoomId::from("0fea69b6-4032-4ea4-9a32-72e818ce11da".to_string()));
    assert_eq!(model.user, UserId::from("0fea69b6-4032-4ea4-9a32-72e818ce11db".to_string()));
    assert_eq!(model.status, true);
}

#[test]
fn kick_user_from_room_test() {
    let mut config = YummyConfig::default();
    config.lua_files_path = temp_dir().to_str().unwrap().to_string();
    
    let path = std::path::Path::new(&config.lua_files_path[..]).join("kick_user_from_room_test.lua").to_string_lossy().to_string();
    let mut lua_file = std::fs::File::create(path).expect("create failed");
    lua_file.write_all(r#"
function pre_kick_user_from_room(model)
    model:set_room("0fea69b6-4032-4ea4-9a32-72e818ce11da")
    model:set_user("0fea69b6-4032-4ea4-9a32-72e818ce11db")
    model:set_ban(true)
end

function post_kick_user_from_room(model, successed)
    assert(model:get_room() == "0fea69b6-4032-4ea4-9a32-72e818ce11da")
    assert(model:get_user() == "0fea69b6-4032-4ea4-9a32-72e818ce11db")
    assert(model:get_ban() == true)
end
"#.as_bytes()).expect("write failed");

    let model = KickUserFromRoom {
        request_id: None,
        auth: Arc::new(None),
        socket: Arc::new(general::test::DummyClient::default()),
        room: RoomId::new(),
        user: UserId::new(),
        ban: false
    };

    let config = Arc::new(config);
    let mut builder = PluginBuilder::default();
    builder.add_installer(Box::new(LuaPluginInstaller::default()));

    let executer = Arc::new(builder.build(config));

    let model = executer.pre_kick_user_from_room(model).unwrap();
    let model = executer.post_kick_user_from_room(model, true).unwrap();

    assert_eq!(model.room, RoomId::from("0fea69b6-4032-4ea4-9a32-72e818ce11da".to_string()));
    assert_eq!(model.user, UserId::from("0fea69b6-4032-4ea4-9a32-72e818ce11db".to_string()));
    assert_eq!(model.ban, true);
}

#[test]
fn disconnect_from_room_request_test() {
    let mut config = YummyConfig::default();
    config.lua_files_path = temp_dir().to_str().unwrap().to_string();
    
    let path = std::path::Path::new(&config.lua_files_path[..]).join("disconnect_from_room_request_test.lua").to_string_lossy().to_string();
    let mut lua_file = std::fs::File::create(path).expect("create failed");
    lua_file.write_all(r#"
function pre_disconnect_from_room_request(model)
    model:set_room("0fea69b6-4032-4ea4-9a32-72e818ce11da")
end

function post_disconnect_from_room_request(model, successed)
    assert(model:get_room() == "0fea69b6-4032-4ea4-9a32-72e818ce11da")
end
"#.as_bytes()).expect("write failed");

    let model = DisconnectFromRoomRequest {
        request_id: None,
        auth: Arc::new(None),
        socket: Arc::new(general::test::DummyClient::default()),
        room: RoomId::new()
    };

    let config = Arc::new(config);
    let mut builder = PluginBuilder::default();
    builder.add_installer(Box::new(LuaPluginInstaller::default()));

    let executer = Arc::new(builder.build(config));

    let model = executer.pre_disconnect_from_room_request(model).unwrap();
    let model = executer.post_disconnect_from_room_request(model, true).unwrap();

    assert_eq!(model.room, RoomId::from("0fea69b6-4032-4ea4-9a32-72e818ce11da".to_string()));
}


#[test]
fn room_list_request_test() {
    let mut config = YummyConfig::default();
    config.lua_files_path = temp_dir().to_str().unwrap().to_string();
    
    let path = std::path::Path::new(&config.lua_files_path[..]).join("room_list_request_test.lua").to_string_lossy().to_string();
    let mut lua_file = std::fs::File::create(path).expect("create failed");
    lua_file.write_all(r#"
function pre_room_list_request(model)
    array = {}
    array[1] = 0
    array[2] = 1
    array[3] = 2
    array[4] = 3

    model:set_members(array)
    model:set_tag("my tag")
end

function post_room_list_request(model, successed)
    model:get_members()
    assert(model:get_tag() == "my tag")
end
"#.as_bytes()).expect("write failed");

    let model = RoomListRequest {
        request_id: None,
        tag: None,
        socket: Arc::new(general::test::DummyClient::default()),
        members: Vec::new()
    };

    let config = Arc::new(config);
    let mut builder = PluginBuilder::default();
    builder.add_installer(Box::new(LuaPluginInstaller::default()));

    let executer = Arc::new(builder.build(config));

    let model = executer.pre_room_list_request(model).unwrap();
    let model = executer.post_room_list_request(model, true).unwrap();

    assert_eq!(model.members, vec![RoomInfoTypeVariant::RoomName, RoomInfoTypeVariant::Description, RoomInfoTypeVariant::Users, RoomInfoTypeVariant::MaxUser]);
    assert_eq!(model.tag, Some("my tag".to_string()));
}


#[test]
fn waiting_room_joins_test() {
    let mut config = YummyConfig::default();
    config.lua_files_path = temp_dir().to_str().unwrap().to_string();
    
    let path = std::path::Path::new(&config.lua_files_path[..]).join("waiting_room_joins_test.lua").to_string_lossy().to_string();
    let mut lua_file = std::fs::File::create(path).expect("create failed");
    lua_file.write_all(r#"
function pre_waiting_room_joins(model)
    model:set_room("0fea69b6-4032-4ea4-9a32-72e818ce11da")
end

function post_waiting_room_joins(model, successed)
    assert(model:get_room() == "0fea69b6-4032-4ea4-9a32-72e818ce11da")
end
"#.as_bytes()).expect("write failed");

    let model = WaitingRoomJoins {
        request_id: None,
        auth: Arc::new(None),
        socket: Arc::new(general::test::DummyClient::default()),
        room: RoomId::new()
    };

    let config = Arc::new(config);
    let mut builder = PluginBuilder::default();
    builder.add_installer(Box::new(LuaPluginInstaller::default()));

    let executer = Arc::new(builder.build(config));

    let model = executer.pre_waiting_room_joins(model).unwrap();
    let model = executer.post_waiting_room_joins(model, true).unwrap();

    assert_eq!(model.room, RoomId::from("0fea69b6-4032-4ea4-9a32-72e818ce11da".to_string()));
}


#[test]
fn get_room_request_test() {
    let mut config = YummyConfig::default();
    config.lua_files_path = temp_dir().to_str().unwrap().to_string();
    
    let path = std::path::Path::new(&config.lua_files_path[..]).join("get_room_request_test.lua").to_string_lossy().to_string();
    let mut lua_file = std::fs::File::create(path).expect("create failed");
    lua_file.write_all(r#"
function pre_get_room_request(model)
    model:set_room("0fea69b6-4032-4ea4-9a32-72e818ce11da")

    array = {}
    array[1] = 0
    array[2] = 1
    array[3] = 2
    array[4] = 3

    model:set_members(array)
end

function post_get_room_request(model, successed)
    model:get_members()
    assert(model:get_room() == "0fea69b6-4032-4ea4-9a32-72e818ce11da")
    assert(model:get_user_id() ~= nil)
    assert(model:get_session_id() ~= nil)

    assert(model:get_user_id() ~= "")
    assert(model:get_session_id() ~= "")
end
"#.as_bytes()).expect("write failed");

    let model = GetRoomRequest {
        request_id: None,
        auth: Arc::new(Some(UserAuth {
            user: UserId::new(),
            session: SessionId::new()
        })),
        socket: Arc::new(general::test::DummyClient::default()),
        members: Vec::new(),
        room: RoomId::new()
    };

    let config = Arc::new(config);
    let mut builder = PluginBuilder::default();
    builder.add_installer(Box::new(LuaPluginInstaller::default()));

    let executer = Arc::new(builder.build(config));

    let model = executer.pre_get_room_request(model).unwrap();
    let model = executer.post_get_room_request(model, true).unwrap();

    assert_eq!(model.members, vec![RoomInfoTypeVariant::RoomName, RoomInfoTypeVariant::Description, RoomInfoTypeVariant::Users, RoomInfoTypeVariant::MaxUser]);
    assert_eq!(model.room, RoomId::from("0fea69b6-4032-4ea4-9a32-72e818ce11da".to_string()));
}
