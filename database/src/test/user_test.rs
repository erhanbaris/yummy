use std::env::temp_dir;

use anyhow::Ok;
use uuid::Uuid;
use crate::SqliteStore;
use crate::model::UserUpdate;
use crate::{create_database, create_connection, PooledConnection};
use crate::auth::*;
use general::meta::*;
use general::model::*;
use crate::user::*;

fn db_conection() -> anyhow::Result<PooledConnection> {
    let mut db_location = temp_dir();
    db_location.push(format!("{}.db", Uuid::new_v4()));

    let mut connection = create_connection(db_location.to_str().unwrap())?.get()?;
    create_database(&mut connection)?;
    Ok(connection)
}

#[test]
fn fail_get_user_information_1() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    assert!(SqliteStore::get_user_information(&mut connection, &UserId::default(), UserMetaAccess::Anonymous)?.is_none());
    Ok(())
}

#[test]
fn fail_get_user_information_2() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    assert!(SqliteStore::get_user_information(&mut connection, &UserId::default(), UserMetaAccess::Anonymous)?.is_none());
    Ok(())
}

#[test]
fn get_user_information_1() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    let user_id = SqliteStore::create_user_via_email(&mut connection, "erhanbaris@gmail.com", &"erhan".into())?;
    let user = SqliteStore::get_user_information(&mut connection, &user_id, UserMetaAccess::Anonymous)?.unwrap();
    assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));
    assert!(user.custom_id.is_none());
    assert!(user.device_id.is_none());
    assert!(user.name.is_none());

    Ok(())
}

#[test]
fn get_user_information_2() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    let user_id = SqliteStore::create_user_via_device_id(&mut connection, "123456789")?;
    let user = SqliteStore::get_user_information(&mut connection, &user_id, UserMetaAccess::Anonymous)?.unwrap();
    assert_eq!(user.device_id, Some("123456789".to_string()));
    assert!(user.email.is_none());
    assert!(user.custom_id.is_none());
    assert!(user.name.is_none());

    Ok(())
}

#[test]
fn get_user_information_3() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    let user_id = SqliteStore::create_user_via_custom_id(&mut connection, "123456789")?;
    let user = SqliteStore::get_user_information(&mut connection, &user_id, UserMetaAccess::Anonymous)?.unwrap();
    assert_eq!(user.custom_id, Some("123456789".to_string()));
    assert!(user.email.is_none());
    assert!(user.device_id.is_none());
    assert!(user.name.is_none());

    Ok(())
}

/* update user tests */
#[test]
fn fail_update_user_1() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    assert!(SqliteStore::update_user(&mut connection, &UserId::default(), &UserUpdate::default()).is_err());
    Ok(())
}

#[test]
fn fail_update_user_2() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    assert!(SqliteStore::update_user(&mut connection, &UserId::default(), &UserUpdate::default()).is_err());
    Ok(())
}
#[test]
fn fail_update_user_3() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    assert_eq!(SqliteStore::update_user(&mut connection, &UserId::default(), &UserUpdate {
        name: Some(Some("123456".to_string())),
        ..Default::default()
    })?, 0);
    Ok(())
}

#[test]
fn fail_update_user_4() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    assert_eq!(SqliteStore::update_user(&mut connection, &UserId::default(), &UserUpdate {
        name: Some(Some("123456".to_string())),
        ..Default::default()
    })?, 0);
    Ok(())
}

#[test]
fn update_user() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    let user_id = SqliteStore::create_user_via_custom_id(&mut connection, "123456789")?;
    assert_eq!(SqliteStore::update_user(&mut connection, &user_id, &UserUpdate {
        name: Some(Some("123456".to_string())),
        ..Default::default()
    })?, 1);
    Ok(())
}

#[test]
fn meta() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    let user_id = SqliteStore::create_user_via_custom_id(&mut connection, "123456789")?;
    assert_eq!(SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::System)?.len(), 0);

    // New meta
    SqliteStore::insert_user_metas(&mut connection, &user_id, vec![(&"gender".to_string(), &MetaType::String("male".to_string(), UserMetaAccess::Friend))])?;

    assert_eq!(SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::Friend)?.len(), 1);
    assert_eq!(SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::Anonymous)?.len(), 0);

    let meta = SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::System)?;
    assert_eq!(meta.len(), 1);

    // Remove meta
    SqliteStore::remove_user_metas(&mut connection, vec![meta[0].0.clone()])?;
    assert_eq!(SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::Friend)?.len(), 0);
    assert_eq!(SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::Anonymous)?.len(), 0);
    assert_eq!(SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::System)?.len(), 0);

    SqliteStore::insert_user_metas(&mut connection, &user_id, vec![
        (&"location".to_string(), &MetaType::String("copenhagen".to_string(), UserMetaAccess::Anonymous)),
        (&"score".to_string(), &MetaType::Number(123.0, UserMetaAccess::Friend))])?;

    assert_eq!(SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::Friend)?.len(), 2);
    assert_eq!(SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::System)?.len(), 2);

    // Filter with anonymous
    let meta = SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::Anonymous)?;
    assert_eq!(meta.len(), 1);
    assert_eq!(meta.into_iter().map(|(_, key, value)| (key, value)).collect::<Vec<(String, MetaType<UserMetaAccess>)>>(), vec![
        ("location".to_string(), MetaType::String("copenhagen".to_string(), UserMetaAccess::Anonymous))]);

    // Filter with system
    let meta = SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::System)?;
    assert_eq!(meta.len(), 2);
    assert_eq!(meta.into_iter().map(|(_, key, value)| (key, value)).collect::<Vec<(String, MetaType<UserMetaAccess>)>>(), vec![
        ("location".to_string(), MetaType::String("copenhagen".to_string(), UserMetaAccess::Anonymous)),
        ("score".to_string(), MetaType::Number(123.0, UserMetaAccess::Friend))]);

    Ok(())
}
