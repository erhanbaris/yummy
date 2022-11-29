#[macro_use]
extern crate diesel;

pub mod auth;
pub mod model;
pub mod user;
pub(crate) mod schema;

use std::str::FromStr;

use auth::AuthStoreTrait;
use diesel::deserialize::FromSql;
use diesel::serialize::IsNull;
use diesel::r2d2::ConnectionManager;
use diesel::SqliteConnection;
use diesel::*;
use diesel::serialize::{ToSql, Output};
use diesel::sql_types::*;
use diesel::expression::AsExpression;

use serde::{Deserialize, Serialize};
use user::UserStoreTrait;
use uuid::Uuid;

pub type Connection = ConnectionManager<SqliteConnection>;
pub type Pool = r2d2::Pool<Connection>;
pub type PooledConnection = ::r2d2::PooledConnection<Connection>;

pub trait DatabaseTrait: AuthStoreTrait + UserStoreTrait + Sized { }
pub struct SqliteStore;

impl DatabaseTrait for SqliteStore { }

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[derive(AsExpression, Copy, Clone, FromSqlRow)]
#[diesel(sql_type = Text)]
pub struct RowId(pub uuid::Uuid);

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

#[derive(SqlType)]
#[derive(Debug, PartialEq, Eq)]
pub enum Visibility {
    Anonymous = 0,
    User = 1,
    Friend = 2,
    Mod = 3,
    Admin = 4,
    System = 5
}

impl ToSql<Integer, diesel::sqlite::Sqlite> for Visibility where i32: ToSql<Integer, diesel::sqlite::Sqlite> {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, diesel::sqlite::Sqlite>) -> serialize::Result {
        out.set_value(match self {
            Visibility::Anonymous => 0,
            Visibility::User => 1,
            Visibility::Friend => 2,
            Visibility::Mod => 3,
            Visibility::Admin => 4,
            Visibility::System => 5,
        });
        Ok(IsNull::No)
    }
}

impl FromSql<Integer, diesel::sqlite::Sqlite> for Visibility where i32: FromSql<Integer, diesel::sqlite::Sqlite> {
    fn from_sql(bytes: backend::RawValue<diesel::sqlite::Sqlite>) -> deserialize::Result<Self> {
        let value = i32::from_ne_bytes(<Vec<u8>>::from_sql(bytes)?[..4].try_into().unwrap());
        Ok(match value {
            0 => Visibility::Anonymous,
            1 => Visibility::User,
            2 => Visibility::Friend,
            3 => Visibility::Mod,
            4 => Visibility::Admin,
            _ => Visibility::System
        })
    }
}

pub fn create_connection(database_url: &str) -> anyhow::Result<Pool> {
    Ok(r2d2::Pool::builder().build(Connection::new(database_url))?)
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
            insert_date INTEGER NOT NULL,
            last_login_date INTEGER NOT NULL
        );
        CREATE TABLE user_metadata (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            key TEXT NOT NULL,
            value TEXT,
            meta_type INTEGER NOT NULL,
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
        assert!(row_id.get().is_nil());

        let row_id = RowId(uuid::Uuid::new_v4());
        assert!(!row_id.get().is_nil());

        let uuid_data = "85fc32fe-eaa5-46c3-b8e8-60bb658b5de7";
        let row_id: RowId = uuid_data.to_string().into();

        let new_uuid_data: String = row_id.to_string();
        assert_eq!(&new_uuid_data, uuid_data);

        Ok(())
    }
}


pub mod exports {
    // we will use that a bit later
    pub use super::Visibility;
}