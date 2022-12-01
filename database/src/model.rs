use std::collections::HashMap;

use crate::{schema::user, schema::user_meta, RowId};
use diesel::*;
use general::meta::MetaType;
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

#[derive(Debug, Insertable)]
#[diesel(table_name = user_meta)]
pub struct UserMetaInsert<'a> {
    pub id: RowId,
    pub user_id: &'a RowId,
    pub key: String,
    pub value: String,
    pub meta_type: i32,
    pub access: i32,
    pub insert_date: i32,
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

#[derive(Default, Clone, Debug, Queryable, Serialize, Deserialize, PartialEq)]
#[diesel(table_name = user)]
pub struct PrivateUserModel {
    pub id: RowId,
    pub name: Option<String>,
    pub email: Option<String>,
    pub device_id: Option<String>,
    pub custom_id: Option<String>,
    pub meta: Option<HashMap<String, MetaType>>,
    pub insert_date: i32,
    pub last_login_date: i32,
}

#[derive(Default, Clone, Debug, Queryable, Serialize, Deserialize, PartialEq)]
#[diesel(table_name = user_meta)]
pub struct UserMetaModel {
    pub id: RowId,
    pub key: String,
    pub value: String,
    pub meta_type: i32,
    pub access: i32,
}
