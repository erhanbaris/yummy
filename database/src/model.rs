use crate::{schema::user, RowId};
use diesel::*;

#[derive(Default, Debug, Insertable)]
#[diesel(table_name = user)]
pub struct UserModel {
    pub id: RowId,
    pub name: Option<String>,
    pub email: Option<String>,
    pub device_id: Option<String>,
    pub password: Option<String>,
    pub insert_date: i32,
    pub last_login_date: i32,
}