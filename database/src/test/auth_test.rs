use anyhow::Ok;

use crate::auth::AuthStoreTrait;
use crate::{create_database, create_connection, PooledConnection};

use crate::SqliteStore;
use super::*;
fn db_conection() -> anyhow::Result<PooledConnection> {
    let mut connection = create_connection(":memory:")?.get()?;
    create_database(&mut connection)?;
    Ok(connection)
}

/* email unit tests */
#[test]
fn create_user_via_email() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    SqliteStore::create_user_via_email(&mut connection, "erhanbaris@gmail.com", &"erhan".into())?;
    Ok(())
}

#[test]
fn login_via_email() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    let created_user_id = SqliteStore::create_user_via_email(&mut connection, "erhanbaris@gmail.com", &"erhan".into())?;
    let result = SqliteStore::user_login_via_email(&mut connection, "erhanbaris@gmail.com")?.unwrap();

    assert_eq!(created_user_id, result.user_id);
    assert!(result.name.is_none());
    assert_eq!(result.password.unwrap_or_default().as_str(), "erhan");

    Ok(())
}

#[test]
fn failed_login_via_email() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    assert!(SqliteStore::user_login_via_email(&mut connection, "erhanbaris@gmail.com")?.is_none());

    Ok(())
}

/* device id unit tests */
#[test]
fn create_user_via_device_id() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    SqliteStore::create_user_via_device_id(&mut connection, "1234567890")?;
    Ok(())
}

#[test]
fn login_via_device_id() -> anyhow::Result<()> {
    let mut connection = db_conection()?;


    let created_user_id = SqliteStore::create_user_via_device_id(&mut connection, "1234567890")?;
    let result = SqliteStore::user_login_via_device_id(&mut connection, "1234567890")?.unwrap();

    assert_eq!(created_user_id, result.user_id);
    assert!(result.name.is_none());
    assert!(result.email.is_none());

    Ok(())
}

#[test]
fn failed_login_via_device_id() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    assert!(SqliteStore::user_login_via_device_id(&mut connection, "1234567890")?.is_none());
    Ok(())
}

/* custom id unit tests */
#[test]
fn create_user_via_custom_id() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    SqliteStore::create_user_via_custom_id(&mut connection, "1234567890")?;
    Ok(())
}

#[test]
fn login_via_custom_id() -> anyhow::Result<()> {
    let mut connection = db_conection()?;


    let created_user_id = SqliteStore::create_user_via_custom_id(&mut connection, "1234567890")?;
    let result = SqliteStore::user_login_via_custom_id(&mut connection, "1234567890")?.unwrap();

    assert_eq!(created_user_id, result.user_id);
    assert!(result.name.is_none());
    assert!(result.email.is_none());

    Ok(())
}

#[test]
fn failed_login_via_custom_id() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    assert!(SqliteStore::user_login_via_custom_id(&mut connection, "1234567890")?.is_none());
    Ok(())
}