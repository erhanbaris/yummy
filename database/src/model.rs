use crate::{schema::user, RowId};
use diesel::*;

#[derive(Default, Debug, Insertable)]
#[diesel(table_name = user)]
pub struct UserModel {
    pub id: RowId,
    pub name: String,
    pub email: String,
    pub password: String,
    pub insert_date: i32,
    pub last_login_date: i32,
}