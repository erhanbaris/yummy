use std::collections::HashMap;

use crate::{schema::user, schema::user_meta, schema::room, schema::room_tag, schema::room_user, RowId};
use diesel::*;
use general::meta::MetaType;
use general::model::UserType;
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
    pub user_type: i32,
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

#[derive(Default, Debug, Insertable)]
#[diesel(table_name = room)]
pub struct RoomInsert<'a> {
    pub id: RowId,
    pub name: Option<String>,
    pub access_type: i32,
    pub password: Option<&'a str>,
    pub max_user: i32,
    pub insert_date: i32,
}

#[derive(Default, Debug, Insertable)]
#[diesel(table_name = room_tag)]
pub struct RoomTagInsert<'a> {
    pub id: RowId,
    pub room_id: RowId,
    pub tag: &'a str,
    pub insert_date: i32,
}

#[derive(Default, Debug, Insertable)]
#[diesel(table_name = room_user)]
pub struct RoomUserInsert {
    pub id: RowId,
    pub room_id: RowId,
    pub user_id: RowId,
    pub room_user_type: i32,
    pub insert_date: i32,
}

#[derive(Default, Debug, AsChangeset)]
#[diesel(table_name = user)]
pub struct UserUpdate {
    pub name: Option<Option<String>>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub user_type: Option<i32>,
    pub device_id: Option<Option<String>>,
    pub custom_id: Option<Option<String>>,
}

#[derive(Default, Clone, Debug, Queryable, Serialize, Deserialize, PartialEq)]
#[diesel(table_name = user)]
pub struct UserInformationModel {
    pub id: RowId,
    pub name: Option<String>,
    pub email: Option<String>,
    pub device_id: Option<String>,
    pub custom_id: Option<String>,
    pub meta: Option<HashMap<String, MetaType>>,
    pub user_type: UserType,
    pub online: bool,
    pub insert_date: i32,
    pub last_login_date: i32,
}

#[derive(Default, Clone, Debug, Queryable, Serialize, Deserialize, PartialEq, Eq)]
#[diesel(table_name = user_meta)]
pub struct UserMetaModel {
    pub id: RowId,
    pub key: String,
    pub value: String,
    pub meta_type: i32,
    pub access: i32,
}

pub struct LoginInfo {
    pub user_id: RowId,
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>
}
