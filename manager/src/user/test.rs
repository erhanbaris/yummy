use actix::Actor;
use actix::Addr;
use anyhow::Ok;
use general::model::UserInformationModel;
use general::model::UserType;
use std::ops::Deref;

use anyhow::anyhow;
use general::auth::UserAuth;
use general::auth::validate_auth;
use general::config::YummyConfig;
use general::config::configure_environment;
use general::config::get_configuration;
use general::meta::MetaAction;
use general::meta::UserMetaAccess;
use general::test::model::AuthenticatedModel;
use general::web::GenericAnswer;
use general::test::DummyClient;
use std::collections::HashMap;
use std::env::temp_dir;
use std::sync::Arc;

use database::{create_database, create_connection};
use general::meta::MetaType;

use crate::auth::AuthManager;
use crate::auth::model::*;
use crate::conn::ConnectionManager;
use crate::plugin::PluginExecuter;

use super::*;

macro_rules! email_auth {
    ($auth_manager: expr, $config: expr, $email: expr, $password: expr, $create: expr, $socket: expr) => {
        {
            $auth_manager.send(EmailAuthRequest {
                request_id: None,
                auth: Arc::new(None),
                email: $email,
                password: $password,
                if_not_exist_create: $create,
                socket: $socket
            }).await??;
        
            let token: AuthenticatedModel = $socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
            let token = token.token;

            let user_jwt = validate_auth($config, token).unwrap().user;
            Arc::new(Some(UserAuth {
                user: user_jwt.id.deref().clone(),
                session: user_jwt.session
            }))
        }
    };
}


fn create_actor() -> anyhow::Result<(Addr<UserManager<database::SqliteStore>>, Addr<AuthManager<database::SqliteStore>>, Arc<YummyConfig>, Arc<DummyClient>)> {
    let mut db_location = temp_dir();
    db_location.push(format!("{}.db", uuid::Uuid::new_v4()));
    
    configure_environment();
    let config = get_configuration();
    #[cfg(feature = "stateless")]
    let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();

    let states = YummyState::new(config.clone(), #[cfg(feature = "stateless")] conn.clone());
    let executer = Arc::new(PluginExecuter::default());

    ConnectionManager::new(config.clone(), states.clone(), executer.clone(), #[cfg(feature = "stateless")] conn.clone()).start();

    let connection = create_connection(db_location.to_str().unwrap())?;
    create_database(&mut connection.clone().get()?)?;
    Ok((UserManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone()), executer.clone()).start(), AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection), executer).start(), config, Arc::new(DummyClient::default())))
}

#[actix::test]
async fn get_private_user_1() -> anyhow::Result<()> {
    let (user_manager, _, _, socket) = create_actor()?;
    assert!(user_manager.send(GetUserInformation::me(None, Arc::new(None), socket.clone())).await?.is_err());
    let message = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    assert!(message.contains("User token is not valid"));
    Ok(())
}

#[actix::test]
async fn get_private_user_2() -> anyhow::Result<()> {
    let (user_manager, auth_manager, config, socket) = create_actor()?;
    auth_manager.send(DeviceIdAuthRequest::new(None, Arc::new(None), "1234567890".to_string(), socket.clone())).await??;
    let token = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    let token: AuthenticatedModel = token.into();
    let token = token.token;

    let user = validate_auth(config, token).unwrap();
    user_manager.send(GetUserInformation::me(None, Arc::new(Some(UserAuth {
        user: user.user.id.deref().clone(),
        session: user.user.session
    })), socket.clone())).await??;

   
    let auth: GenericAnswer<UserInformationModel> = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let user = auth.result;

    assert_eq!(user.device_id, Some("1234567890".to_string()));
    Ok(())
}

#[actix::test]
async fn fail_update_get_user_1() -> anyhow::Result<()> {
    let (user_manager, _, _, socket) = create_actor()?;
    let result = user_manager.send(UpdateUser {
        auth: Arc::new(None),
        socket,
        ..Default::default()
    }).await?;
    assert!(result.is_err());
    Ok(())
}

#[actix::test]
async fn fail_update_get_user_2() -> anyhow::Result<()> {
    let (user_manager, auth_manager, config, socket) = create_actor()?;
    auth_manager.send(DeviceIdAuthRequest::new(None, Arc::new(None), "1234567890".to_string(), socket.clone())).await??;
    let token = socket.clone().messages.lock().unwrap().pop_back().unwrap();
    let token: AuthenticatedModel = token.into();
    let token = token.token;


    let user = validate_auth(config, token).unwrap();
    let result = user_manager.send(UpdateUser {
        auth: Arc::new(Some(UserAuth {
            user: user.user.id.deref().clone(),
            session: user.user.session
        })),
        socket,
        ..Default::default()
    }).await?;
    assert!(result.is_err());
    Ok(())
}

#[actix::test]
async fn fail_update_get_user_3() -> anyhow::Result<()> {
    let (user_manager, auth_manager, config, socket) = create_actor()?;
    auth_manager.send(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        email: "erhanbaris@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: true,
        socket: socket.clone()
    }).await??;
    
    let token: AuthenticatedModel = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let token = token.token;

    let user = validate_auth(config, token).unwrap();
    let result = user_manager.send(UpdateUser {
        auth: Arc::new(Some(UserAuth {
            user: user.user.id.deref().clone(),
            session: user.user.session
        })),
        socket,
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
    let (user_manager, auth_manager, config, socket) = create_actor()?;
    auth_manager.send(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        email: "erhanbaris@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: true,
        socket: socket.clone()
    }).await??;
    
    let token: AuthenticatedModel = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let token = token.token;

    let user = validate_auth(config, token).unwrap();
    let result = user_manager.send(UpdateUser {
        auth: Arc::new(Some(UserAuth {
            user: user.user.id.deref().clone(),
            session: user.user.session
        })),
        socket,
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
    let (user_manager, auth_manager, config, socket) = create_actor()?;
    auth_manager.send(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        email: "erhanbaris@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: true,
        socket: socket.clone()
    }).await??;

    let token: AuthenticatedModel = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let token = token.token;

    let user_jwt = validate_auth(config, token).unwrap().user;
    let user_auth = Arc::new(Some(UserAuth {
        user: user_jwt.id.deref().clone(),
        session: user_jwt.session
    }));
    
    let result = user_manager.send(UpdateUser {
        auth: user_auth.clone(),
        password: Some("123".to_string()),
        socket,
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
    let (user_manager, auth_manager, config, socket) = create_actor()?;

    auth_manager.send(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        email: "erhanbaris@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: true,
        socket: socket.clone()
    }).await??;

    let token: AuthenticatedModel = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let token = token.token;

    let user_jwt = validate_auth(config, token).unwrap().user;
    let user_auth = Arc::new(Some(UserAuth {
        user: user_jwt.id.deref().clone(),
        session: user_jwt.session
    }));

    let result = user_manager.send(UpdateUser {
        auth: user_auth.clone(),
        email: Some("erhanbaris@gmail.com".to_string()),
        socket,
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
    let (user_manager, auth_manager, config, socket) = create_actor()?;

    auth_manager.send(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        email: "erhanbaris@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: true,
        socket: socket.clone()
    }).await??;

    let token: AuthenticatedModel = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let token = token.token;

    let user_jwt = validate_auth(config, token).unwrap().user;
    let user_auth = Arc::new(Some(UserAuth {
        user: user_jwt.id.deref().clone(),
        session: user_jwt.session
    }));

    user_manager.send(GetUserInformation::me(None, user_auth.clone(), socket.clone())).await??;

    let auth: GenericAnswer<UserInformationModel> = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let user = auth.result;
    
    assert_eq!(user.name, None);
    assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));
    
    user_manager.send(UpdateUser {
        auth: user_auth.clone(),
        name: Some("Erhan".to_string()),
        socket: socket.clone(),
        ..Default::default()
    }).await??;

    user_manager.send(GetUserInformation::me(None, user_auth.clone(), socket.clone())).await??;
    
    let auth: GenericAnswer<UserInformationModel> = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let user = auth.result;

    assert_eq!(user.name, Some("Erhan".to_string()));
    assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));

    Ok(())
}

#[actix::test]
async fn update_user_2() -> anyhow::Result<()> {
    let (user_manager, auth_manager, config, socket) = create_actor()?;
    let admin = email_auth!(auth_manager, config.clone(), "admin@gmail.com".to_string(), "erhan".into(), true, socket.clone());
    
    user_manager.send(UpdateUser {
        auth: admin.clone(),
        user_type : Some(UserType::Admin),
        socket: socket.clone(),
        ..Default::default()
    }).await??;

    auth_manager.send(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        email: "erhanbaris@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: true,
        socket: socket.clone()
    }).await??;
    
    let token: AuthenticatedModel = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let token = token.token;

    let user_jwt = validate_auth(config, token).unwrap().user;
    let user_auth = Arc::new(Some(UserAuth {
        user: user_jwt.id.deref().clone(),
        session: user_jwt.session
    }));

    user_manager.send(GetUserInformation::me(None, user_auth.clone(), socket.clone())).await??;

    let auth: GenericAnswer<UserInformationModel> = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let user = auth.result;
    
    assert_eq!(user.name, None);
    assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));
    
    user_manager.send(UpdateUser {
        auth: admin.clone(),
        target_user_id: Some(user_jwt.id.deref().clone()),
        socket: socket.clone(),
        name: Some("Erhan".to_string()),
        metas: Some(HashMap::from([
            ("gender".to_string(), MetaType::String("Male".to_string(), UserMetaAccess::Friend)),
            ("location".to_string(), MetaType::String("Copenhagen".to_string(), UserMetaAccess::Friend)),
            ("postcode".to_string(), MetaType::Number(1000.0, UserMetaAccess::Mod)),
            ("score".to_string(), MetaType::Number(15.3, UserMetaAccess::Anonymous)),
            ("temp_admin".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
        ])),
        ..Default::default()
    }).await??;

    user_manager.send(GetUserInformation::me(None, user_auth.clone(), socket.clone())).await??;

    let auth: GenericAnswer<UserInformationModel> = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let user = auth.result;

    assert_eq!(user.name, Some("Erhan".to_string()));
    assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));

    /* Cleanup fields */
    user_manager.send(UpdateUser {
        auth: user_auth.clone(),
        name: Some("Erhan".to_string()),
        socket: socket.clone(),
        ..Default::default()
    }).await??;

    user_manager.send(GetUserInformation::me(None, user_auth.clone(), socket.clone())).await??;

    let auth: GenericAnswer<UserInformationModel> = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let user = auth.result;

    assert_eq!(user.name, Some("Erhan".to_string()));
    assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));

    Ok(())
}

#[actix::test]
async fn update_user_3() -> anyhow::Result<()> {
    let (user_manager, auth_manager, config, socket) = create_actor()?;
    let admin = email_auth!(auth_manager, config.clone(), "admin@gmail.com".to_string(), "erhan".into(), true, socket.clone());
    
    user_manager.send(UpdateUser {
        auth: admin.clone(),
        user_type : Some(UserType::Admin),
        socket: socket.clone(),
        ..Default::default()
    }).await??;

    auth_manager.send(EmailAuthRequest {
        request_id: None,
        auth: Arc::new(None),
        email: "erhanbaris@gmail.com".to_string(),
        password:"erhan".into(),
        if_not_exist_create: true,
        socket: socket.clone()
    }).await??;

    let token: AuthenticatedModel = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let token = token.token;

    let user_jwt = validate_auth(config, token).unwrap().user;
    let user_auth = Arc::new(Some(UserAuth {
        user: user_jwt.id.deref().clone(),
        session: user_jwt.session
    }));

    user_manager.send(GetUserInformation::me(None, user_auth.clone(), socket.clone())).await??;
    let auth: GenericAnswer<UserInformationModel> = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let user = auth.result;
    
    assert_eq!(user.name, None);
    assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));

    /*Max meta must be 10, this request is valid */
    user_manager.send(UpdateUser {
        auth: admin.clone(),
        target_user_id: Some(user_jwt.id.deref().clone()),
        name: Some("Erhan".to_string()),
        socket: socket.clone(),
        metas: Some(HashMap::from([
            ("1".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
            ("2".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
            ("3".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
            ("4".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
            ("5".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
            ("6".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
            ("7".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
            ("8".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
            ("9".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
            ("10".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
        ])),
        ..Default::default()
    }).await??;
    
    /*Max meta must be 10, this request is NOT valid */
    let response = user_manager.send(UpdateUser {
        auth: admin.clone(),
        target_user_id: Some(user_jwt.id.deref().clone()),
        name: Some("Erhan".to_string()),
        socket: socket.clone(),
        metas: Some(HashMap::from([
            ("1".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
            ("2".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
            ("3".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
            ("4".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
            ("5".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
            ("6".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
            ("7".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
            ("8".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
            ("9".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
            ("10".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
            ("11".to_string(), MetaType::Bool(true, UserMetaAccess::Admin)),
        ])),
        ..Default::default()
    }).await?;

    assert!(response.is_err());

    Ok(())
}

#[actix::test]
async fn meta_manupulation_test_1() -> anyhow::Result<()> {
    let (user_manager, auth_manager, config, socket) = create_actor()?;

    let admin = email_auth!(auth_manager, config.clone(), "admin@gmail.com".to_string(), "erhan".into(), true, socket.clone());
    let moderator = email_auth!(auth_manager, config.clone(), "moderator@gmail.com".to_string(), "erhan".into(), true, socket.clone());
    let user = email_auth!(auth_manager, config.clone(), "user@gmail.com".to_string(), "erhan".into(), true, socket.clone());
    let other_user = email_auth!(auth_manager, config, "other_user@gmail.com".to_string(), "erhan".into(), true, socket.clone());

    let user_id = match user.as_ref() {
        Some(user) => user.user.clone(),
        None => return Err(anyhow!("User not found"))
    };

    user_manager.send(UpdateUser {
        auth: admin.clone(),
        user_type : Some(UserType::Admin),
        socket: socket.clone(),
        ..Default::default()
    }).await??;
    
    user_manager.send(UpdateUser {
        auth: moderator.clone(),
        user_type : Some(UserType::Mod),
        socket: socket.clone(),
        ..Default::default()
    }).await??;

    /* Update user's meta information to test */
    user_manager.send(UpdateUser {
        auth: admin.clone(),
        target_user_id: Some(user_id.clone()),
        metas: Some(HashMap::from([
            //("system".to_string(), MetaType::Number(112233.0, UserMetaAccess::System)),
            ("admin".to_string(), MetaType::Number(123456789.0, UserMetaAccess::Admin)),
            ("moderator".to_string(), MetaType::String("Copennhagen".to_string(), UserMetaAccess::Mod)),
            ("me".to_string(), MetaType::Bool(true, UserMetaAccess::Me)),
            ("friend".to_string(), MetaType::String("123".to_string(), UserMetaAccess::Friend)),
            ("user".to_string(), MetaType::String("88".to_string(), UserMetaAccess::User)),
            ("anonymous".to_string(), MetaType::String("99".to_string(), UserMetaAccess::Anonymous)),
        ])),
        ..Default::default()
    }).await??;

    /* Check for my informations */
    user_manager.send(GetUserInformation::me(None, user.clone(), socket.clone())).await??;
    
    let information: GenericAnswer<UserInformationModel> = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let information = information.result;

    assert!(information.metas.is_some());
    let information_meta = information.metas.unwrap();
    assert_eq!(information_meta.len(), 4);
    assert_eq!(information_meta.get("anonymous"), Some(&MetaType::String("99".to_string(), UserMetaAccess::Anonymous)));
    assert_eq!(information_meta.get("user"), Some(&MetaType::String("88".to_string(), UserMetaAccess::Anonymous)));
    assert_eq!(information_meta.get("friend"), Some(&MetaType::String("123".to_string(), UserMetaAccess::Anonymous)));
    assert_eq!(information_meta.get("me"), Some(&MetaType::Bool(true, UserMetaAccess::Anonymous)));

    /* Check for moderator */
    user_manager.send(GetUserInformation::user(None, user_id.clone(), moderator.clone(), socket.clone())).await??;
    
    let information: GenericAnswer<UserInformationModel> = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let information = information.result;

    assert!(information.metas.is_some());
    let information_meta = information.metas.unwrap();
    assert_eq!(information_meta.len(), 5);
    assert_eq!(information_meta.get("anonymous"), Some(&MetaType::String("99".to_string(), UserMetaAccess::Anonymous)));
    assert_eq!(information_meta.get("user"), Some(&MetaType::String("88".to_string(), UserMetaAccess::Anonymous)));
    assert_eq!(information_meta.get("friend"), Some(&MetaType::String("123".to_string(), UserMetaAccess::Anonymous)));
    assert_eq!(information_meta.get("me"), Some(&MetaType::Bool(true, UserMetaAccess::Anonymous)));
    assert_eq!(information_meta.get("moderator"), Some(&MetaType::String("Copennhagen".to_string(), UserMetaAccess::Anonymous)));


    /* Check for admin */
    user_manager.send(GetUserInformation::user(None, user_id.clone(), admin.clone(), socket.clone())).await??;
    let information: GenericAnswer<UserInformationModel> = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let information = information.result;

    assert!(information.metas.is_some());
    let information_meta = information.metas.unwrap();
    assert_eq!(information_meta.len(), 6);
    assert_eq!(information_meta.get("anonymous"), Some(&MetaType::String("99".to_string(), UserMetaAccess::Anonymous)));
    assert_eq!(information_meta.get("user"), Some(&MetaType::String("88".to_string(), UserMetaAccess::Anonymous)));
    assert_eq!(information_meta.get("friend"), Some(&MetaType::String("123".to_string(), UserMetaAccess::Anonymous)));
    assert_eq!(information_meta.get("me"), Some(&MetaType::Bool(true, UserMetaAccess::Anonymous)));
    assert_eq!(information_meta.get("moderator"), Some(&MetaType::String("Copennhagen".to_string(), UserMetaAccess::Anonymous)));
    assert_eq!(information_meta.get("admin"), Some(&MetaType::Number(123456789.0, UserMetaAccess::Anonymous)));

    /* Check for system */
    user_manager.send(GetUserInformation::user_via_system(None, user_id.clone(), socket.clone())).await??;
    let information: GenericAnswer<UserInformationModel> = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let information = information.result;

    assert!(information.metas.is_some());
    let information_meta = information.metas.unwrap();
    assert_eq!(information_meta.len(), 6);
    assert_eq!(information_meta.get("anonymous"), Some(&MetaType::String("99".to_string(), UserMetaAccess::Anonymous)));
    assert_eq!(information_meta.get("user"), Some(&MetaType::String("88".to_string(), UserMetaAccess::Anonymous)));
    assert_eq!(information_meta.get("friend"), Some(&MetaType::String("123".to_string(), UserMetaAccess::Anonymous)));
    assert_eq!(information_meta.get("me"), Some(&MetaType::Bool(true, UserMetaAccess::Anonymous)));
    assert_eq!(information_meta.get("moderator"), Some(&MetaType::String("Copennhagen".to_string(), UserMetaAccess::Anonymous)));
    assert_eq!(information_meta.get("admin"), Some(&MetaType::Number(123456789.0, UserMetaAccess::Anonymous)));
    //assert_eq!(information_meta.get("system"), Some(&MetaType::Number(112233.0, UserMetaAccess::Anonymous)));

    /* Check for other user */
    user_manager.send(GetUserInformation::user(None, user_id.clone(), other_user.clone(), socket.clone())).await??;
    let information: GenericAnswer<UserInformationModel> = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let information = information.result;

    assert!(information.metas.is_some());
    let information_meta = information.metas.unwrap();
    assert_eq!(information_meta.len(), 2);
    assert_eq!(information_meta.get("anonymous"), Some(&MetaType::String("99".to_string(), UserMetaAccess::Anonymous)));
    assert_eq!(information_meta.get("user"), Some(&MetaType::String("88".to_string(), UserMetaAccess::Anonymous)));

    /* Check for anonymous */
    user_manager.send(GetUserInformation::user(None, user_id.clone(), Arc::new(None), socket.clone())).await??;
    let information: GenericAnswer<UserInformationModel> = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let information = information.result;

    assert!(information.metas.is_some());
    let information_meta = information.metas.unwrap();
    assert_eq!(information_meta.len(), 1);
    assert_eq!(information_meta.get("anonymous"), Some(&MetaType::String("99".to_string(), UserMetaAccess::Anonymous)));

    Ok(())
}

#[actix::test]
async fn meta_manupulation_test_2() -> anyhow::Result<()> {
    let (user_manager, auth_manager, config, socket) = create_actor()?;
    let user = email_auth!(auth_manager, config.clone(), "user@gmail.com".to_string(), "erhan".into(), true, socket.clone());

    user_manager.send(UpdateUser {
        auth: user.clone(),
        name: Some("Erhan".to_string()),
        socket: socket.clone(),
        metas: Some(HashMap::from([
            ("1".to_string(), MetaType::Bool(true, UserMetaAccess::Me)),
            ("2".to_string(), MetaType::Bool(true, UserMetaAccess::Me)),
            ("3".to_string(), MetaType::Bool(true, UserMetaAccess::Me)),
            ("4".to_string(), MetaType::Bool(true, UserMetaAccess::Me)),
            ("5".to_string(), MetaType::Bool(true, UserMetaAccess::Me)),
            ("6".to_string(), MetaType::Bool(true, UserMetaAccess::Me)),
            ("7".to_string(), MetaType::Bool(true, UserMetaAccess::Me)),
            ("8".to_string(), MetaType::Bool(true, UserMetaAccess::Me)),
            ("9".to_string(), MetaType::Bool(true, UserMetaAccess::Me)),
            ("10".to_string(), MetaType::Bool(true, UserMetaAccess::Me)),
        ])),
        ..Default::default()
    }).await??;

    // Remove unused metas
    user_manager.send(UpdateUser {
        auth: user.clone(),
        name: Some("Erhan".to_string()),
        socket: socket.clone(),
        metas: Some(HashMap::from([
            ("1".to_string(), MetaType::Bool(true, UserMetaAccess::Me)),
            ("2".to_string(), MetaType::Bool(true, UserMetaAccess::Me)),
            ("3".to_string(), MetaType::Bool(true, UserMetaAccess::Me)),
            ("4".to_string(), MetaType::Bool(true, UserMetaAccess::Me)),
            ("5".to_string(), MetaType::Bool(true, UserMetaAccess::Me)),
        ])),
        meta_action: Some(MetaAction::RemoveUnusedMetas),
        ..Default::default()
    }).await??;

    /* Check for my informations */
    user_manager.send(GetUserInformation::me(None, user.clone(), socket.clone())).await??;
    
    let information: GenericAnswer<UserInformationModel> = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let information = information.result;

    assert!(information.metas.is_some());
    let information_meta = information.metas.unwrap();
    assert_eq!(information_meta.len(), 5);



    // Remove unused metas
    user_manager.send(UpdateUser {
        auth: user.clone(),
        name: Some("Erhan".to_string()),
        socket: socket.clone(),
        metas: Some(HashMap::from([
            ("6".to_string(), MetaType::Bool(true, UserMetaAccess::Me)),
        ])),
        meta_action: Some(MetaAction::OnlyAddOrUpdate),
        ..Default::default()
    }).await??;

    /* Check for my informations */
    user_manager.send(GetUserInformation::me(None, user.clone(), socket.clone())).await??;
    
    let information: GenericAnswer<UserInformationModel> = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let information = information.result;

    assert!(information.metas.is_some());
    let information_meta = information.metas.unwrap();
    assert_eq!(information_meta.len(), 6);



    // Remove single meta
    user_manager.send(UpdateUser {
        auth: user.clone(),
        name: Some("Erhan".to_string()),
        socket: socket.clone(),
        metas: Some(HashMap::from([
            ("6".to_string(), MetaType::Null),
        ])),
        ..Default::default()
    }).await??;

    /* Check for my informations */
    user_manager.send(GetUserInformation::me(None, user.clone(), socket.clone())).await??;
    
    let information: GenericAnswer<UserInformationModel> = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let information = information.result;

    assert!(information.metas.is_some());
    let information_meta = information.metas.unwrap();
    assert_eq!(information_meta.len(), 5);



    // Remove all metas
    user_manager.send(UpdateUser {
        auth: user.clone(),
        name: Some("Erhan".to_string()),
        socket: socket.clone(),
        meta_action: Some(MetaAction::RemoveAllMetas),
        ..Default::default()
    }).await??;


    /* Check for my informations */
    user_manager.send(GetUserInformation::me(None, user.clone(), socket.clone())).await??;
    
    let information: GenericAnswer<UserInformationModel> = socket.clone().messages.lock().unwrap().pop_back().unwrap().into();
    let information = information.result;

    assert!(information.metas.is_none());

    Ok(())  
}
