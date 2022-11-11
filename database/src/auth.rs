use chrono::Utc;
use diesel::*;
use secrecy::{SecretString, ExposeSecret};
use uuid::Uuid;

use crate::{PooledConnection, schema::user, RowId, model::UserModel};

pub struct AuthStore {
    database: PooledConnection
}

pub trait AuthStoreTrait {
    fn new(database: PooledConnection) -> Self;
    fn user_login_via_email(&mut self, email: &str) -> Result<Option<(RowId, Option<String>, SecretString)>, crate::error::Error>;
    fn create_user_via_email(&mut self, email: &str, password: &SecretString) -> Result<RowId, crate::error::Error>;
}

impl AuthStoreTrait for AuthStore {
    fn new(database: PooledConnection) -> Self {
        Self { database }
    }

    #[tracing::instrument(name="User login via email", skip(self))]
    fn user_login_via_email(&mut self, email: &str) -> Result<Option<(RowId, Option<String>, SecretString)>, crate::error::Error> {
        let result = user::table
            .filter(user::email.eq(email))
            .select((user::id, user::name, user::password))
            .first::<(RowId, String, String)>(&mut *self.database)
            .optional()?
            .map(|(id, name, password)| (id, Some(name), SecretString::new(password)));
        tracing::info!("{:?}", result);
        Ok(result)
    }

    #[tracing::instrument(name="User login via email", skip(self))]
    fn create_user_via_email(&mut self, email: &str, password: &SecretString) -> Result<RowId, crate::error::Error> {
        
        let row_id = RowId(Uuid::new_v4());
        let mut model = UserModel::default();
        model.id = row_id;
        model.insert_date = Utc::now().timestamp() as i32;
        model.password = password.expose_secret().to_string();
        model.email = email.to_string();
        diesel::insert_into(user::table).values(&vec![model]).execute(&mut *self.database)?;
        
        tracing::info!("{:?}", row_id);
        Ok(row_id)
    }
}