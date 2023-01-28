use std::{rc::Rc, cell::RefCell, sync::Arc};

use general::password::Password;

use crate::plugin::EmailAuthRequest;
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

    plugin.execute(model.clone(), "post_email_auth").unwrap();
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

    plugin.execute(model.clone(), "post_email_auth").unwrap_err();
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
    plugin.execute(model.clone(), "post_email_auth").unwrap();
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
    plugin.execute(model.clone(), "post_email_auth").unwrap();

    assert!(Rc::strong_count(&model) <= 3); // This is not good, but, I dont have a solution yet
}
