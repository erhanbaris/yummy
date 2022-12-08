use actix::Recipient;
use anyhow::anyhow;
use general::auth::UserAuth;
use general::auth::validate_auth;
use general::config::YummyConfig;
use general::config::get_configuration;
use general::meta::MetaAccess;
use general::model::WebsocketMessage;
use general::model::YummyState;
use std::collections::HashMap;
use std::env::temp_dir;
use std::sync::Arc;

use actix::Actor;
use actix::Addr;
use anyhow::Ok;
use database::{create_database, create_connection};

use super::*;
use crate::api::auth::AuthManager;
use crate::api::auth::model::*;

use actix::{Context, Handler};
struct dummy_actor;

impl Actor for dummy_actor {
    type Context = Context<Self>;
}

impl Handler<WebsocketMessage> for dummy_actor {
    type Result = ();
    
    fn handle(&mut self, _: WebsocketMessage, _: &mut Self::Context) { }
}

macro_rules! email_auth {
    ($auth_manager: expr, $config: expr, $email: expr, $password: expr, $create: expr, $recipient: expr) => {
        {
            let token = $auth_manager.send(EmailAuthRequest {
                email: $email,
                password: $password,
                if_not_exist_create: $create,
                socket: $recipient
            }).await??;
        
            let token = match token {
                Response::Auth(token, _) => token,
                _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
            };
        
            let user_jwt = validate_auth($config, token).unwrap().user;
            Arc::new(Some(UserAuth {
                user: user_jwt.id,
                session: user_jwt.session
            }))
        }
    };
}


fn create_actor() -> anyhow::Result<(Addr<UserManager<database::SqliteStore>>, Addr<AuthManager<database::SqliteStore>>, Arc<YummyConfig>, Recipient<WebsocketMessage>)> {
    let mut db_location = temp_dir();
    db_location.push(format!("{}.db", Uuid::new_v4()));
    
    let config = get_configuration();
    let states = Arc::new(YummyState::default());
    let connection = create_connection(db_location.to_str().unwrap())?;
    create_database(&mut connection.clone().get()?)?;
    Ok((UserManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone())).start(), AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection)).start(), config, dummy_actor {}.start().recipient()))
}

#[actix::test]
async fn get_private_user_1() -> anyhow::Result<()> {
    let (user_manager, _, _, _) = create_actor()?;
    let user = user_manager.send(GetUserInformation::me(Arc::new(None))).await?;
    assert!(user.is_err());
    Ok(())
}

#[actix::test]
async fn get_private_user_2() -> anyhow::Result<()> {
    let (user_manager, auth_manager, config, recipient) = create_actor()?;
    let token = auth_manager.send(DeviceIdAuthRequest::new("1234567890".to_string(), recipient)).await??;
    let token = match token {
        Response::Auth(token, _) => token,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
    };

    let user = validate_auth(config, token).unwrap();
    let user = user_manager.send(GetUserInformation::me(Arc::new(Some(UserAuth {
        user: user.user.id,
        session: user.user.session
    })))).await??;

    let user = match user {
        Response::UserInformation(model) => model,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::UserInformation'")); }
    };

    assert_eq!(user.device_id, Some("1234567890".to_string()));
    Ok(())
}

#[actix::test]
async fn fail_update_get_user_1() -> anyhow::Result<()> {
    let (user_manager, _, _, _) = create_actor()?;
    let result = user_manager.send(UpdateUser {
        user: Arc::new(None),
        ..Default::default()
    }).await?;
    assert!(result.is_err());
    Ok(())
}

#[actix::test]
async fn fail_update_get_user_2() -> anyhow::Result<()> {
    let (user_manager, auth_manager, config, recipient) = create_actor()?;
    let token = auth_manager.send(DeviceIdAuthRequest::new("1234567890".to_string(), recipient)).await??;
    let token = match token {
        Response::Auth(token, _) => token,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
    };

    let user = validate_auth(config, token).unwrap();
    let result = user_manager.send(UpdateUser {
        user: Arc::new(Some(UserAuth {
            user: user.user.id,
            session: user.user.session
        })),
        ..Default::default()
    }).await?;
    assert!(result.is_err());
    Ok(())
}

#[actix::test]
async fn fail_update_get_user_3() -> anyhow::Result<()> {
    let (user_manager, auth_manager, config, recipient) = create_actor()?;
    let token = auth_manager.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password: "erhan".to_string(),
        if_not_exist_create: true,
        socket: recipient
    }).await??;
    
    let token = match token {
        Response::Auth(token, _) => token,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
    };

    let user = validate_auth(config, token).unwrap();
    let result = user_manager.send(UpdateUser {
        user: Arc::new(Some(UserAuth {
            user: user.user.id,
            session: user.user.session
        })),
        email: Some("erhanbaris@gmail.com".to_string()),
        ..Default::default()
    }).await?;

    match result {
        std::result::Result::Ok(_) => { assert!(false, "Expected 'CannotChangeEmail' error message"); },
        Err(error) => { assert_eq!(error.downcast_ref::<UserError>().unwrap(), &UserError::CannotChangeEmail); }
    };

    Ok(())
}

#[actix::test]
async fn fail_update_get_user_4() -> anyhow::Result<()> {
    let (user_manager, auth_manager, config, recipient) = create_actor()?;
    let token = auth_manager.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password: "erhan".to_string(),
        if_not_exist_create: true,
        socket: recipient
    }).await??;
    
    let token = match token {
        Response::Auth(token, _) => token,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
    };

    let user = validate_auth(config, token).unwrap();
    let result = user_manager.send(UpdateUser {
        user: Arc::new(Some(UserAuth {
            user: user.user.id,
            session: user.user.session
        })),
        ..Default::default()
    }).await?;

    match result {
        std::result::Result::Ok(_) => { assert!(false, "Expected 'UpdateInformationMissing' error message"); },
        Err(error) => { assert_eq!(error.downcast_ref::<UserError>().unwrap(), &UserError::UpdateInformationMissing); }
    };

    Ok(())
}

#[actix::test]
async fn fail_update_password() -> anyhow::Result<()> {
    let (user_manager, auth_manager, config, recipient) = create_actor()?;
    let token = auth_manager.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password: "erhan".to_string(),
        if_not_exist_create: true,
        socket: recipient
    }).await??;

    let token = match token {
        Response::Auth(token, _) => token,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
    };

    let user_jwt = validate_auth(config, token).unwrap().user;
    let user_auth = Arc::new(Some(UserAuth {
        user: user_jwt.id,
        session: user_jwt.session
    }));
    
    let result = user_manager.send(UpdateUser {
        user: user_auth.clone(),
        password: Some("123".to_string()),
        ..Default::default()
    }).await?;

    match result {
        std::result::Result::Ok(_) => { assert!(false, "Expected 'PasswordIsTooSmall' error message"); },
        Err(error) => { assert_eq!(error.downcast_ref::<UserError>().unwrap(), &UserError::PasswordIsTooSmall); }
    };

    Ok(())
}

#[actix::test]
async fn fail_update_email() -> anyhow::Result<()> {
    let (user_manager, auth_manager, config, recipient) = create_actor()?;

    let token = auth_manager.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password: "erhan".to_string(),
        if_not_exist_create: true,
        socket: recipient
    }).await??;

    let token = match token {
        Response::Auth(token, _) => token,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
    };

    let user_jwt = validate_auth(config, token).unwrap().user;
    let user_auth = Arc::new(Some(UserAuth {
        user: user_jwt.id,
        session: user_jwt.session
    }));

    let result = user_manager.send(UpdateUser {
        user: user_auth.clone(),
        email: Some("erhanbaris@gmail.com".to_string()),
        ..Default::default()
    }).await?;

    match result {
        std::result::Result::Ok(_) => { assert!(false, "Expected 'CannotChangeEmail' error message"); },
        Err(error) => { assert_eq!(error.downcast_ref::<UserError>().unwrap(), &UserError::CannotChangeEmail); }
    };

    Ok(())
}

#[actix::test]
async fn update_user_1() -> anyhow::Result<()> {
    let (user_manager, auth_manager, config, recipient) = create_actor()?;

    let token = auth_manager.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password: "erhan".to_string(),
        if_not_exist_create: true,
        socket: recipient
    }).await??;

    let token = match token {
        Response::Auth(token, _) => token,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
    };

    let user_jwt = validate_auth(config, token).unwrap().user;
    let user_auth = Arc::new(Some(UserAuth {
        user: user_jwt.id,
        session: user_jwt.session
    }));

    let user = user_manager.send(GetUserInformation::me(user_auth.clone())).await??;

    let user = match user {
        Response::UserInformation(model) => model,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::UserInformation'")); }
    };
    
    assert_eq!(user.name, None);
    assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));
    
    user_manager.send(UpdateUser {
        user: user_auth.clone(),
        name: Some("Erhan".to_string()),
        ..Default::default()
    }).await??;

    let user = user_manager.send(GetUserInformation::me(user_auth.clone())).await??;

    let user = match user {
        Response::UserInformation(model) => model,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::UserInformation'")); }
    };

    assert_eq!(user.name, Some("Erhan".to_string()));
    assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));

    Ok(())
}

#[actix::test]
async fn update_user_2() -> anyhow::Result<()> {
    let (user_manager, auth_manager, config, recipient) = create_actor()?;

    let token = auth_manager.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password: "erhan".to_string(),
        if_not_exist_create: true,
        socket: recipient
    }).await??;

    let token = match token {
        Response::Auth(token, _) => token,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
    };

    let user_jwt = validate_auth(config, token).unwrap().user;
    let user_auth = Arc::new(Some(UserAuth {
        user: user_jwt.id,
        session: user_jwt.session
    }));

    let user = user_manager.send(GetUserInformation::me(user_auth.clone())).await??;

    let user = match user {
        Response::UserInformation(model) => model,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::UserInformation'")); }
    };
    
    assert_eq!(user.name, None);
    assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));
    
    user_manager.send(UpdateUser {
        user: user_auth.clone(),
        name: Some("Erhan".to_string()),
        meta: Some(HashMap::from([
            ("gender".to_string(), MetaType::String("Male".to_string(), MetaAccess::Friend)),
            ("location".to_string(), MetaType::String("Copenhagen".to_string(), MetaAccess::Friend)),
            ("postcode".to_string(), MetaType::Number(1000.0, MetaAccess::Mod)),
            ("score".to_string(), MetaType::Number(15.3, MetaAccess::Anonymous)),
            ("temp_admin".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
        ])),
        ..Default::default()
    }).await??;

    let user = user_manager.send(GetUserInformation::me(user_auth.clone())).await??;

    let user = match user {
        Response::UserInformation(model) => model,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::UserInformation'")); }
    };

    assert_eq!(user.name, Some("Erhan".to_string()));
    assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));

    /* Cleanup fields */
    user_manager.send(UpdateUser {
        user: user_auth.clone(),
        name: Some("Erhan".to_string()),
        ..Default::default()
    }).await??;

    let user = user_manager.send(GetUserInformation::me(user_auth.clone())).await??;

    let user = match user {
        Response::UserInformation(model) => model,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::UserInformation'")); }
    };

    assert_eq!(user.name, Some("Erhan".to_string()));
    assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));

    Ok(())
}

#[actix::test]
async fn update_user_3() -> anyhow::Result<()> {
    let (user_manager, auth_manager, config, recipient) = create_actor()?;

    let token = auth_manager.send(EmailAuthRequest {
        email: "erhanbaris@gmail.com".to_string(),
        password: "erhan".to_string(),
        if_not_exist_create: true,
        socket: recipient
    }).await??;

    let token = match token {
        Response::Auth(token, _) => token,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::Auth'")); }
    };

    let user_jwt = validate_auth(config, token).unwrap().user;
    let user_auth = Arc::new(Some(UserAuth {
        user: user_jwt.id,
        session: user_jwt.session
    }));

    let user = user_manager.send(GetUserInformation::me(user_auth.clone())).await??;

    let user = match user {
        Response::UserInformation(model) => model,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::UserInformation'")); }
    };
    
    assert_eq!(user.name, None);
    assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));

    /*Max meta must be 10, this request is valid */
    user_manager.send(UpdateUser {
        user: user_auth.clone(),
        name: Some("Erhan".to_string()),
        meta: Some(HashMap::from([
            ("1".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ("2".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ("3".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ("4".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ("5".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ("6".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ("7".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ("8".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ("9".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ("10".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
        ])),
        ..Default::default()
    }).await??;
    
    /*Max meta must be 10, this request is NOT valid */
    let response = user_manager.send(UpdateUser {
        user: user_auth.clone(),
        name: Some("Erhan".to_string()),
        meta: Some(HashMap::from([
            ("1".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ("2".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ("3".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ("4".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ("5".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ("6".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ("7".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ("8".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ("9".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ("10".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
            ("11".to_string(), MetaType::Bool(true, MetaAccess::Admin)),
        ])),
        ..Default::default()
    }).await?;

    assert!(response.is_err());

    Ok(())
}


#[actix::test]
async fn meta_manupulation_test() -> anyhow::Result<()> {
    let (user_manager, auth_manager, config, recipient) = create_actor()?;

    let admin = email_auth!(auth_manager, config.clone(), "admin@gmail.com".to_string(), "erhan".to_string(), true, recipient.clone());
    let moderator = email_auth!(auth_manager, config.clone(), "moderator@gmail.com".to_string(), "erhan".to_string(), true, recipient.clone());
    let user = email_auth!(auth_manager, config.clone(), "user@gmail.com".to_string(), "erhan".to_string(), true, recipient.clone());
    let other_user = email_auth!(auth_manager, config, "other_user@gmail.com".to_string(), "erhan".to_string(), true, recipient);

    let user_id = match user.as_ref() {
        Some(user) => user.user.clone(),
        None => return Err(anyhow!("User not found"))
    };

    user_manager.send(UpdateUser {
        user: admin.clone(),
        user_type : Some(UserType::Admin),
        ..Default::default()
    }).await??;
    
    user_manager.send(UpdateUser {
        user: moderator.clone(),
        user_type : Some(UserType::Mod),
        ..Default::default()
    }).await??;

    /* Update user's meta information to test */
    user_manager.send(UpdateUser {
        user: user.clone(),
        meta: Some(HashMap::from([
            ("system".to_string(), MetaType::Number(112233.0, MetaAccess::System)),
            ("admin".to_string(), MetaType::Number(123456789.0, MetaAccess::Admin)),
            ("moderator".to_string(), MetaType::String("Copennhagen".to_string(), MetaAccess::Mod)),
            ("me".to_string(), MetaType::Bool(true, MetaAccess::Me)),
            ("friend".to_string(), MetaType::String("123".to_string(), MetaAccess::Friend)),
            ("user".to_string(), MetaType::String("88".to_string(), MetaAccess::User)),
            ("anonymous".to_string(), MetaType::String("99".to_string(), MetaAccess::Anonymous)),
        ])),
        ..Default::default()
    }).await??;

    /* Check for my informations */
    let information = user_manager.send(GetUserInformation::me(user.clone())).await??;
    let information = match information {
        Response::UserInformation(model) => model,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::UserInformation'")); }
    };

    assert!(information.meta.is_some());
    let information_meta = information.meta.unwrap();
    assert_eq!(information_meta.len(), 4);
    assert_eq!(information_meta.get("anonymous"), Some(&MetaType::String("99".to_string(), MetaAccess::Anonymous)));
    assert_eq!(information_meta.get("user"), Some(&MetaType::String("88".to_string(), MetaAccess::User)));
    assert_eq!(information_meta.get("friend"), Some(&MetaType::String("123".to_string(), MetaAccess::Friend)));
    assert_eq!(information_meta.get("me"), Some(&MetaType::Bool(true, MetaAccess::Me)));

    /* Check for moderator */
    let information = user_manager.send(GetUserInformation::user(user_id.clone(), moderator.clone())).await??;
    let information = match information {
        Response::UserInformation(model) => model,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::UserInformation'")); }
    };

    assert!(information.meta.is_some());
    let information_meta = information.meta.unwrap();
    assert_eq!(information_meta.len(), 5);
    assert_eq!(information_meta.get("anonymous"), Some(&MetaType::String("99".to_string(), MetaAccess::Anonymous)));
    assert_eq!(information_meta.get("user"), Some(&MetaType::String("88".to_string(), MetaAccess::User)));
    assert_eq!(information_meta.get("friend"), Some(&MetaType::String("123".to_string(), MetaAccess::Friend)));
    assert_eq!(information_meta.get("me"), Some(&MetaType::Bool(true, MetaAccess::Me)));
    assert_eq!(information_meta.get("moderator"), Some(&MetaType::String("Copennhagen".to_string(), MetaAccess::Mod)));


    /* Check for admin */
    let information = user_manager.send(GetUserInformation::user(user_id.clone(), admin.clone())).await??;
    let information = match information {
        Response::UserInformation(model) => model,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::UserInformation'")); }
    };

    assert!(information.meta.is_some());
    let information_meta = information.meta.unwrap();
    assert_eq!(information_meta.len(), 6);
    assert_eq!(information_meta.get("anonymous"), Some(&MetaType::String("99".to_string(), MetaAccess::Anonymous)));
    assert_eq!(information_meta.get("user"), Some(&MetaType::String("88".to_string(), MetaAccess::User)));
    assert_eq!(information_meta.get("friend"), Some(&MetaType::String("123".to_string(), MetaAccess::Friend)));
    assert_eq!(information_meta.get("me"), Some(&MetaType::Bool(true, MetaAccess::Me)));
    assert_eq!(information_meta.get("moderator"), Some(&MetaType::String("Copennhagen".to_string(), MetaAccess::Mod)));
    assert_eq!(information_meta.get("admin"), Some(&MetaType::Number(123456789.0, MetaAccess::Admin)));

    /* Check for system */
    let information = user_manager.send(GetUserInformation::user_via_system(user_id.clone())).await??;
    let information = match information {
        Response::UserInformation(model) => model,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::UserInformation'")); }
    };

    assert!(information.meta.is_some());
    let information_meta = information.meta.unwrap();
    assert_eq!(information_meta.len(), 7);
    assert_eq!(information_meta.get("anonymous"), Some(&MetaType::String("99".to_string(), MetaAccess::Anonymous)));
    assert_eq!(information_meta.get("user"), Some(&MetaType::String("88".to_string(), MetaAccess::User)));
    assert_eq!(information_meta.get("friend"), Some(&MetaType::String("123".to_string(), MetaAccess::Friend)));
    assert_eq!(information_meta.get("me"), Some(&MetaType::Bool(true, MetaAccess::Me)));
    assert_eq!(information_meta.get("moderator"), Some(&MetaType::String("Copennhagen".to_string(), MetaAccess::Mod)));
    assert_eq!(information_meta.get("admin"), Some(&MetaType::Number(123456789.0, MetaAccess::Admin)));
    assert_eq!(information_meta.get("system"), Some(&MetaType::Number(112233.0, MetaAccess::System)));

    /* Check for other user */
    let information = user_manager.send(GetUserInformation::user(user_id.clone(), other_user.clone())).await??;
    let information = match information {
        Response::UserInformation(model) => model,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::UserInformation'")); }
    };

    assert!(information.meta.is_some());
    let information_meta = information.meta.unwrap();
    assert_eq!(information_meta.len(), 2);
    assert_eq!(information_meta.get("anonymous"), Some(&MetaType::String("99".to_string(), MetaAccess::Anonymous)));
    assert_eq!(information_meta.get("user"), Some(&MetaType::String("88".to_string(), MetaAccess::User)));

    /* Check for anonymous */
    let information = user_manager.send(GetUserInformation::user(user_id.clone(), Arc::new(None))).await??;
    let information = match information {
        Response::UserInformation(model) => model,
        _ => { return Err(anyhow::anyhow!("Expected 'Response::UserInformation'")); }
    };

    assert!(information.meta.is_some());
    let information_meta = information.meta.unwrap();
    assert_eq!(information_meta.len(), 1);
    assert_eq!(information_meta.get("anonymous"), Some(&MetaType::String("99".to_string(), MetaAccess::Anonymous)));

    Ok(())
}
