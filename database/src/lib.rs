#[macro_use]
extern crate diesel;

pub mod auth;
pub mod model;
pub mod user;
pub mod room;
pub(crate) mod schema;

use std::str::FromStr;

use auth::AuthStoreTrait;
use diesel::deserialize::FromSql;
use diesel::serialize::IsNull;
use diesel::r2d2::ConnectionManager;
use diesel::SqliteConnection;
use diesel::*;
use diesel::Connection;
use diesel::serialize::{ToSql, Output};
use diesel::sql_types::*;
use diesel::expression::AsExpression;

use serde::{Deserialize, Serialize};
use user::UserStoreTrait;
use room::RoomStoreTrait;
use uuid::Uuid;

pub type ConnectionType = SqliteConnection;
pub type Pool = r2d2::Pool<ConnectionManager<ConnectionType>>;
pub type PooledConnection = ::r2d2::PooledConnection<ConnectionManager<ConnectionType>>;

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

impl DatabaseTrait for SqliteStore { }

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[derive(AsExpression, Copy, Clone, FromSqlRow)]
#[diesel(sql_type = Text)]
pub struct RowId(pub uuid::Uuid);

impl Default for RowId {
    fn default() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl RowId {
    pub fn get(&self) -> Uuid {
        self.0
    }
}

impl ToString for RowId {
    fn to_string(&self) -> String {
        self.get().to_string()
    }
}

impl From<Uuid> for RowId {
    fn from(id: Uuid) -> Self {
        RowId(id)
    }
}

impl ToSql<Text, diesel::sqlite::Sqlite> for RowId where String: ToSql<Text, diesel::sqlite::Sqlite> {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, diesel::sqlite::Sqlite>) -> serialize::Result {
        out.set_value(self.get().to_string());
        Ok(IsNull::No)
    }
}

impl FromSql<Text, diesel::sqlite::Sqlite> for RowId where String: FromSql<Text, diesel::sqlite::Sqlite> {
    fn from_sql(bytes: backend::RawValue<diesel::sqlite::Sqlite>) -> deserialize::Result<Self> {
        let value = String::from_utf8(<Vec<u8>>::from_sql(bytes)?)?;
        let row_id = Uuid::from_str(&value)?;
        Ok(RowId(row_id))
    }
}

impl From<String> for RowId {
    fn from(data: String) -> Self {
        RowId(Uuid::from_str(&data).unwrap_or_default())
    }
}

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
            max_user INTEGER NOT NULL,
            password TEXT,
            access_type INTEGER NOT NULL,
            access_supplementary TEXT,
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
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::RowId;

    #[test]
    fn row_id() -> anyhow::Result<()> {
        let row_id = RowId::default();
        assert!(!row_id.get().is_nil());

        let row_id = RowId(uuid::Uuid::new_v4());
        assert!(!row_id.get().is_nil());

        let uuid_data = "85fc32fe-eaa5-46c3-b8e8-60bb658b5de7";
        let row_id: RowId = uuid_data.to_string().into();

        let new_uuid_data: String = row_id.to_string();
        assert_eq!(&new_uuid_data, uuid_data);

        Ok(())
    }
}
