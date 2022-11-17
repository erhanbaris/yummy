use crate::{schema::user, RowId};
use diesel::*;

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
pub struct UserUpdate<'a> {
    pub name: Option<Option<&'a str>>,
    pub email: Option<&'a str>,
    pub device_id: Option<Option<&'a str>>,
    pub custom_id: Option<Option<&'a str>>,
    pub password: Option<Option<&'a str>>,
}

#[derive(Default, Debug, Queryable)]
#[diesel(table_name = user)]
pub struct PrivateUserModel {
    pub name: Option<String>,
    pub email: Option<String>,
    pub device_id: Option<String>,
    pub custom_id: Option<String>,
    pub insert_date: i32,
    pub last_login_date: i32,
}