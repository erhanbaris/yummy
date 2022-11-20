use crate::{schema::user, RowId};
use diesel::*;
use serde::Serialize;
use serde::Deserialize;

#[derive(Default, Debug, Insertable)]
#[diesel(table_name = user)]
pub struct UserInsert<'a> {
    pub id: RowId,
    pub name: Option<&'a str>,
    pub email: Option<&'a str>,
    pub device_id: Option<&'a str>,
    pub custom_id: Option<&'a str>,
    pub password: Option<&'a str>,
    pub insert_date: i32,
    pub last_login_date: i32,
}

#[derive(Default, Debug, AsChangeset)]
#[diesel(table_name = user)]
pub struct UserUpdate {
    pub name: Option<Option<String>>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub device_id: Option<Option<String>>,
    pub custom_id: Option<Option<String>>,
}

#[derive(Default, Debug, Queryable, Serialize, Deserialize, PartialEq)]
#[diesel(table_name = user)]
pub struct PrivateUserModel {
    pub id: RowId,
    pub name: Option<String>,
    pub email: Option<String>,
    pub device_id: Option<String>,
    pub custom_id: Option<String>,
    pub insert_date: i32,
    pub last_login_date: i32,
}

#[derive(Default, Debug, Queryable, Serialize, Deserialize, PartialEq)]
#[diesel(table_name = user)]
pub struct PublicUserModel {
    pub id: RowId,
    pub name: Option<String>,
    pub last_login_date: i32,
}
