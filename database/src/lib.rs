#[macro_use]
extern crate diesel;

pub mod auth;
pub mod model;
pub mod user;
pub mod room;
pub(crate) mod schema;

#[cfg(test)]
mod test;

use auth::AuthStoreTrait;
use diesel::r2d2::ConnectionManager;
use diesel::*;

use user::UserStoreTrait;
use room::RoomStoreTrait;

use general::database::{PooledConnection, Pool, ConnectionType};

pub trait DatabaseTrait: AuthStoreTrait + UserStoreTrait + RoomStoreTrait + Sized {
    fn transaction<T, E, F>(connection: &mut PooledConnection, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut PooledConnection) -> Result<T, E>,
        E: From<anyhow::Error> + std::convert::From<diesel::result::Error>,
    {
        connection.transaction(f)
    }
}

pub struct SqliteStore;

pub type DefaultDatabaseStore = SqliteStore;

impl DatabaseTrait for SqliteStore { }

pub fn create_connection(database_url: &str) -> anyhow::Result<Pool> {
    Ok(r2d2::Pool::builder().build(ConnectionManager::<ConnectionType>::new(database_url))?)
}

pub fn create_database(connection: &mut PooledConnection) -> anyhow::Result<()> {
    sql_query(
        r#"CREATE TABLE user (
            id TEXT PRIMARY KEY,
            name TEXT UNIQUE,
            device_id TEXT UNIQUE,
            custom_id TEXT UNIQUE,
            email TEXT UNIQUE,
            password TEXT,
            user_type INTEGER,
            insert_date INTEGER NOT NULL,
            last_login_date INTEGER NOT NULL
        );"#,
    )
    .execute(connection)?;
    sql_query(
        r#"
        CREATE TABLE user_meta (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            meta_type INTEGER NOT NULL,
            access INTEGER NOT NULL,
            insert_date INTEGER NOT NULL
        );"#,
    )
    .execute(connection)?;
    sql_query(
        r#"
        CREATE TABLE room (
            id TEXT PRIMARY KEY,
            name TEXT,
            description TEXT,
            max_user INTEGER NOT NULL,
            password TEXT,
            access_type INTEGER NOT NULL,
            join_request INTEGER NOT NULL,
            insert_date INTEGER NOT NULL
        );"#,
    )
    .execute(connection)?;
    sql_query(
        r#"
        CREATE TABLE room_tag (
            id TEXT PRIMARY KEY,
            room_id TEXT NOT NULL,
            tag name TEXT,
            insert_date INTEGER NOT NULL
        );"#,
    )
    .execute(connection)?;
    sql_query(
        r#"
        CREATE TABLE room_user (
            id TEXT PRIMARY KEY,
            room_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            room_user_type TEXT NOT NULL,
            insert_date INTEGER NOT NULL
        );"#,
    )
    .execute(connection)?;
    sql_query(
        r#"
        CREATE TABLE room_user_request (
            id TEXT PRIMARY KEY,
            room_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            room_user_type TEXT NOT NULL,
            status TEXT NOT NULL,
            status_updater_user_id TEXT,
            insert_date INTEGER NOT NULL
        );"#,
    )
    .execute(connection)?;
    sql_query(
        r#"
        CREATE TABLE room_user_ban (
            id TEXT PRIMARY KEY,
            room_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            blocker_user_id TEXT,
            insert_date INTEGER NOT NULL
        );"#,
    )
    .execute(connection)?;
    sql_query(
        r#"
        CREATE TABLE room_meta (
            id TEXT PRIMARY KEY,
            room_id TEXT NOT NULL,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            meta_type INTEGER NOT NULL,
            access INTEGER NOT NULL,
            insert_date INTEGER NOT NULL
        );"#,
    )
    .execute(connection)?;
    Ok(())
}
