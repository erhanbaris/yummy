use chrono::Utc;
use diesel::*;
use secrecy::{SecretString, ExposeSecret};
use uuid::Uuid;

use crate::{PooledConnection, schema::user, RowId, model::UserModel};

pub struct AuthStore {
    database: PooledConnection
}

impl AuthStore {
    pub fn new(database: PooledConnection) -> Self {
        Self { database }
    }
}

impl AuthStore {
    #[tracing::instrument(name="User login via email", skip(self))]
    pub fn user_login_via_email(&mut self, email: &str) -> Result<Option<(RowId, SecretString)>, crate::error::Error> {
        let result = user::table
            .filter(user::email.eq(email))
            .select((user::id, user::password))
            .first::<(RowId, String)>(&mut *self.database)
            .optional()?
            .map(|(id, password)| (id, SecretString::new(password)));
        Ok(result)
    }

    #[tracing::instrument(name="User login via email", skip(self))]
    pub fn create_user_via_email(&mut self, email: &str, password: &SecretString) -> Result<RowId, crate::error::Error> {
        
        let row_id = RowId(Uuid::new_v4());
        let mut model = UserModel::default();
        model.id = row_id;
        model.insert_date = Utc::now().timestamp() as i32;
        model.password = password.expose_secret().to_string();
        model.email = email.to_string();
        diesel::insert_into(user::table).values(&vec![model]).execute(&mut *self.database)?;
        
        Ok(row_id)
    }
}