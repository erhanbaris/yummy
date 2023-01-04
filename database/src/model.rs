use std::collections::HashMap;

use crate::{schema::user, schema::user_meta, schema::room, schema::room_tag, schema::room_user, schema::room_meta};
use diesel::*;
use general::meta::MetaType;
use general::meta::UserMetaAccess;
use general::model::RoomId;
use general::model::RoomMetaId;
use general::model::RoomTagId;
use general::model::RoomUserId;
use general::model::UserId;
use general::model::UserMetaId;
use general::model::UserType;
use serde::Serialize;
use serde::Deserialize;

#[derive(Debug, Insertable)]
#[diesel(table_name = user)]
pub struct UserInsert<'a> {
    pub id: &'a UserId,
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
    pub id: UserMetaId,
    pub user_id: &'a UserId,
    pub key: String,
    pub value: String,
    pub meta_type: i32,
    pub access: i32,
    pub insert_date: i32,
}

#[derive(Default, Debug, Insertable)]
#[diesel(table_name = room)]
pub struct RoomInsert {
    pub id: RoomId,
    pub name: Option<String>,
    pub access_type: i32,
    pub max_user: i32,
    pub insert_date: i32,
}

#[derive(Default, Debug, AsChangeset)]
#[diesel(table_name = room)]
pub struct RoomUpdate {
    pub name: Option<Option<String>>,
    pub max_user: Option<i32>,
    pub access_type: Option<i32>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = room_tag)]
pub struct RoomTagInsert<'a> {
    pub id: RoomTagId,
    pub room_id: &'a RoomId,
    pub tag: &'a str,
    pub insert_date: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = room_user)]
pub struct RoomUserInsert<'a> {
    pub id: RoomUserId,
    pub room_id: &'a RoomId,
    pub user_id: &'a UserId,
    pub room_user_type: i32,
    pub insert_date: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = room_meta)]
pub struct RoomMetaInsert<'a> {
    pub id: UserMetaId,
    pub room_id: &'a RoomId,
    pub key: &'a String,
    pub value: String,
    pub meta_type: i32,
    pub access: i32,
    pub insert_date: i32,
}

#[derive(Default, Clone, Debug, Queryable, Serialize, Deserialize, PartialEq, Eq)]
#[diesel(table_name = room_meta)]
pub struct RoomMetaModel {
    pub id: RoomMetaId,
    pub key: String,
    pub value: String,
    pub meta_type: i32,
    pub access: i32,
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
    pub id: UserId,
    pub name: Option<String>,
    pub email: Option<String>,
    pub device_id: Option<String>,
    pub custom_id: Option<String>,
    pub meta: Option<HashMap<String, MetaType<UserMetaAccess>>>,
    pub user_type: UserType,
    pub online: bool,
    pub insert_date: i32,
    pub last_login_date: i32,
}

#[derive(Default, Clone, Debug, Queryable, Serialize, Deserialize, PartialEq, Eq)]
#[diesel(table_name = user_meta)]
pub struct UserMetaModel {
    pub id: UserMetaId,
    pub key: String,
    pub value: String,
    pub meta_type: i32,
    pub access: i32,
}

pub struct LoginInfo {
    pub user_id: UserId,
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub user_type: UserType
}
