use crate::{schema::user, RowId};
use diesel::*;

#[derive(Default, Debug, Insertable)]
#[diesel(table_name = user)]
pub struct UserModel<'a> {
    pub id: RowId,
    pub name: Option<&'a str>,
    pub email: Option<&'a str>,
    pub device_id: Option<&'a str>,
    pub password: Option<&'a str>,
    pub insert_date: i32,
    pub last_login_date: i32,
}