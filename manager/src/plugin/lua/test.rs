use std::{rc::Rc, cell::RefCell, sync::Arc, env::temp_dir};
use std::io::Write;

use general::meta::{MetaType, UserMetaAccess};
use general::model::UserId;
use general::{password::Password, config::YummyConfig};

use crate::auth::model::ConnUserDisconnect;
use crate::conn::model::UserConnected;
use crate::user::model::{GetUserInformation, GetUserInformationEnum, UpdateUser};
use crate::{plugin::{EmailAuthRequest, PluginBuilder, lua::LuaPluginInstaller}, auth::model::{DeviceIdAuthRequest, CustomIdAuthRequest, LogoutRequest, RefreshTokenRequest, RestoreTokenRequest}};
use super::LuaPlugin;

#[test]
fn executest_1() -> anyhow::Result<()> {
    let plugin = LuaPlugin::new();
    plugin.execute(Rc::new(RefCell::new(EmailAuthRequest {
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
    config.lua_files_path = temp_dir().to_str().unwrap().to_string();
    
    let path = std::path::Path::new(&config.lua_files_path[..]).join("create_meta_test.lua").to_string_lossy().to_string();
    let mut lua_file = std::fs::File::create(path).expect("create failed");
    lua_file.write_all(r#"
function do_tables_match( a, b )
    return table.concat(a) == table.concat(b)
end

function pre_update_user(model)
    model:set_email("erhan@erhan.com")
    metas = {}
    metas.string_value = new_user_meta("Test", 6)
    metas.number_value = new_user_meta(123456, 5)
    metas.bool_value = new_user_meta(true, 4)
    metas.nil_value = new_user_meta(nil, 3)

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
    assert(metas.nil_value:get_value() == nil)

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

    assert(model:get_email() == "erhan@erhan.com")
end
"#.as_bytes()).expect("write failed");

    let model = UpdateUser {
        auth: Arc::new(None),
        email: None,
        password: None,
        socket: Arc::new(general::test::DummyClient::default()),
        target_user_id: None,
        name: None,
        device_id: None,
        custom_id: None,
        user_type: None,
        meta: None,
        meta_action: None,
    };

    let config = Arc::new(config);
    let mut builder = PluginBuilder::default();
    builder.add_installer(Box::new(LuaPluginInstaller::default()));

    let executer = Arc::new(builder.build(config));

    let model = executer.pre_update_user(model).unwrap();
    let model = executer.post_update_user(model, true).unwrap();
    
    assert_eq!(model.email, Some("erhan@erhan.com".to_string()));
    let metas = model.meta.unwrap();
    assert_eq!(metas.len(), 5);
    assert_eq!(metas.get("string_value").unwrap(), &MetaType::String("Test".to_string(), UserMetaAccess::System));
    assert_eq!(metas.get("number_value").unwrap(), &MetaType::Number(123456.0, UserMetaAccess::Admin));
    assert_eq!(metas.get("bool_value").unwrap(), &MetaType::Bool(true, UserMetaAccess::Mod));
    assert_eq!(metas.get("nil_value").unwrap(), &MetaType::Null);
    assert_eq!(metas.get("array_value").unwrap(), &MetaType::List(Box::new(vec![MetaType::Bool(true, UserMetaAccess::Anonymous), MetaType::String("2".to_string(), UserMetaAccess::Anonymous), MetaType::Number(3.0, UserMetaAccess::Anonymous), MetaType::List(Box::new(Vec::new()), UserMetaAccess::Anonymous)]), UserMetaAccess::Me));
}
