#[macro_use]
extern crate diesel;

pub mod auth;
pub mod error;
pub mod model;
pub(crate) mod schema;

use std::str::FromStr;

use diesel::deserialize::FromSql;
use diesel::serialize::IsNull;
use diesel::r2d2::ConnectionManager;
use diesel::SqliteConnection;
use diesel::*;
use diesel::serialize::{ToSql, Output};
use diesel::sql_types::*;
use diesel::expression::AsExpression;

use error::Error;
use uuid::Uuid;

pub type Connection = ConnectionManager<SqliteConnection>;
pub type Pool = r2d2::Pool<Connection>;
pub type PooledConnection = ::r2d2::PooledConnection<Connection>;

sql_function! {fn last_insert_rowid() -> Text;}

#[derive(Debug, Default)]
#[derive(AsExpression, Copy, Clone, FromSqlRow)]
#[diesel(sql_type = Text)]
pub struct RowId(pub uuid::Uuid);

impl ToString for RowId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<Uuid> for RowId {
    fn from(id: Uuid) -> Self {
        RowId(id)
    }
}

impl From<RowId> for Uuid {
    fn from(row: RowId) -> Self {
        row.0.clone()
    }
}

impl ToSql<Text, diesel::sqlite::Sqlite> for RowId where String: ToSql<Text, diesel::sqlite::Sqlite> {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, diesel::sqlite::Sqlite>) -> serialize::Result {
        out.set_value(self.0.to_string());
        Ok(IsNull::No)
    }
}

impl FromSql<Text, diesel::sqlite::Sqlite> for RowId where String: FromSql<Text, diesel::sqlite::Sqlite> {
    fn from_sql(bytes: backend::RawValue<diesel::sqlite::Sqlite>) -> deserialize::Result<Self> {
        let value = <Vec<u8>>::from_sql(bytes)?;
        let row_id = Uuid::from_slice(&value)?;
        Ok(RowId(row_id))
    }
}

impl From<String> for RowId {
    fn from(data: String) -> Self {
        RowId(Uuid::from_str(&data).unwrap_or_default())
    }
}

pub fn create_connection(database_url: &str) -> Result<Pool, Error> {
    Ok(r2d2::Pool::builder().build(Connection::new(database_url))?)
}

pub fn get_last_insert_id(connection: &mut PooledConnection) -> Result<RowId, Error> {
    let row_id = select(last_insert_rowid())
        .first::<String>(connection)
        .map(|str| Uuid::from_str(&str))?
        .map(|uuid| RowId(uuid))?;
    Ok(row_id)
}

pub fn create_database(connection: &mut PooledConnection) -> Result<(), Error> {
    sql_query(
        r#"CREATE TABLE user (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL,
            insert_date INTEGER NOT NULL,
            last_login_date INTEGER NOT NULL
        );"#,
    )
    .execute(connection)?;
    Ok(())
}
