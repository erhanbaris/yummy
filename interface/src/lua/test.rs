use std::{rc::Rc, cell::RefCell, sync::Arc};

use general::password::Password;

use crate::auth::YummyEmailAuthModel;

use super::{LuaYummyAuthPlugin, model::CallbackType};

#[test]
fn executest_1() -> anyhow::Result<()> {
    let plugin = LuaYummyAuthPlugin::new();
    plugin.execute(Rc::new(RefCell::new(YummyEmailAuthModel {
        ref_id: 0,
        auth: Arc::new(None),
        email: "".to_string(),
        password: Password::from("123456".to_string()),
        if_not_exist_create: false,
        socket: Arc::new(general::test::DummyClient::default())
    } )), CallbackType::PreEmailAuth)?;
    Ok(())
}

#[test]
#[should_panic]
fn lua_code_empty() {
    let mut plugin = LuaYummyAuthPlugin::new();
    plugin.add(CallbackType::PreEmailAuth, "").unwrap();
}

#[test]
#[should_panic]
fn lua_code_has_problem() {
    let mut plugin = LuaYummyAuthPlugin::new();
    plugin.add(CallbackType::PreEmailAuth, r#"
j = 10 123
"#).unwrap();
}

#[test]
#[should_panic]
fn no_lua_function() {
    let mut plugin = LuaYummyAuthPlugin::new();
    plugin.add(CallbackType::PreEmailAuth, r#"
j = 123
"#).unwrap();
}

#[test]
fn valid_lua_function() {
    let mut plugin = LuaYummyAuthPlugin::new();
    plugin.add(CallbackType::PreEmailAuth, r#"
function(model)
end
"#).unwrap();

plugin.execute(Rc::new(RefCell::new(YummyEmailAuthModel {
        ref_id: 0,
        auth: Arc::new(None),
        email: "".to_string(),
        password: Password::from("123456".to_string()),
        if_not_exist_create: false,
        socket: Arc::new(general::test::DummyClient::default())
    } )), CallbackType::PreEmailAuth).unwrap();
}

#[test]
fn change_email_address() {
    let model = Rc::new(RefCell::new(YummyEmailAuthModel {
        ref_id: 0,
        auth: Arc::new(None),
        email: "".to_string(),
        password: Password::from("123456".to_string()),
        if_not_exist_create: false,
        socket: Arc::new(general::test::DummyClient::default())
    } ));

    let mut plugin = LuaYummyAuthPlugin::new();
    plugin.add(CallbackType::PreEmailAuth, r#"
    function(model)
        model:get_ref_id()
        model:get_user_id()
        model:get_session_id()
        model:get_password()
        model:get_create()

        model:set_email("erhan@erhan.com")
        model:set_password("SECRET")
        model:set_create(true)
    end
    "#).unwrap();

    plugin.execute(model.clone(), CallbackType::PreEmailAuth).unwrap();
    assert_eq!(&model.borrow().email, "erhan@erhan.com");
    assert_eq!(model.borrow().password.get(), "SECRET");
    assert_eq!(model.borrow().if_not_exist_create, true);
}