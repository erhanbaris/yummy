use chrono::Utc;
use diesel::*;
use uuid::Uuid;

use crate::{PooledConnection, schema::user, RowId, model::UserModel};

pub struct AuthStore {
    database: PooledConnection
}

pub trait AuthStoreTrait {
    fn new(database: PooledConnection) -> Self;

    fn user_login_via_email(&mut self, email: &str) -> Result<Option<(RowId, Option<String>, String)>, crate::error::Error>;
    fn user_login_via_device_id(&mut self, device_id: &str) -> Result<Option<(RowId, Option<String>, Option<String>)>, crate::error::Error>;
    fn user_login_via_custom_id(&mut self, custom_id: &str) -> Result<Option<(RowId, Option<String>, Option<String>)>, crate::error::Error>;

    fn create_user_via_email(&mut self, email: &str, password: &str) -> Result<RowId, crate::error::Error>;
    fn create_user_via_device_id(&mut self, device_id: &str) -> Result<RowId, crate::error::Error>;
    fn create_user_via_custom_id(&mut self, custom_id: &str) -> Result<RowId, crate::error::Error>;
}

impl AuthStoreTrait for AuthStore {
    fn new(database: PooledConnection) -> Self {
        Self { database }
    }

    #[tracing::instrument(name="User login via email", skip(self))]
    fn user_login_via_email(&mut self, email: &str) -> Result<Option<(RowId, Option<String>, String)>, crate::error::Error> {
        let result = user::table
            .filter(user::email.eq(email))
            .select((user::id, user::name, user::password))
            .first::<(RowId, Option<String>, Option<String>)>(&mut *self.database)
            .optional()?
            .map(|(id, name, password)| (id, name, password.unwrap_or_default()));

        Ok(result)
    }

    #[tracing::instrument(name="User login via device id", skip(self))]
    fn user_login_via_device_id(&mut self, device_id: &str) -> Result<Option<(RowId, Option<String>, Option<String>)>, crate::error::Error> {
        let result = user::table
            .filter(user::device_id.eq(device_id))
            .select((user::id, user::name, user::email))
            .first::<(RowId, Option<String>, Option<String>)>(&mut *self.database)
            .optional()?;

        Ok(result)
    }

    #[tracing::instrument(name="User login via custom id", skip(self))]
    fn user_login_via_custom_id(&mut self, custom_id: &str) -> Result<Option<(RowId, Option<String>, Option<String>)>, crate::error::Error> {
        let result = user::table
            .filter(user::custom_id.eq(custom_id))
            .select((user::id, user::name, user::email))
            .first::<(RowId, Option<String>, Option<String>)>(&mut *self.database)
            .optional()?;

        Ok(result)
    }

    #[tracing::instrument(name="User create via email", skip(self))]
    fn create_user_via_email(&mut self, email: &str, password: &str) -> Result<RowId, crate::error::Error> {
        
        let row_id = RowId(Uuid::new_v4());
        let mut model = UserModel::default();
        model.id = row_id;
        model.insert_date = Utc::now().timestamp() as i32;
        model.password = Some(password);
        model.email = Some(email);
        diesel::insert_into(user::table).values(&vec![model]).execute(&mut *self.database)?;

        Ok(row_id)
    }

    #[tracing::instrument(name="User create via device id", skip(self))]
    fn create_user_via_device_id(&mut self, device_id: &str) -> Result<RowId, crate::error::Error> {
        let row_id = RowId(Uuid::new_v4());
        let mut model = UserModel::default();
        model.id = row_id;
        model.insert_date = Utc::now().timestamp() as i32;
        model.device_id = Some(device_id);
        diesel::insert_into(user::table).values(&vec![model]).execute(&mut *self.database)?;

        Ok(row_id)
    }

    #[tracing::instrument(name="User create via custom id", skip(self))]
    fn create_user_via_custom_id(&mut self, custom_id: &str) -> Result<RowId, crate::error::Error> {
        let row_id = RowId(Uuid::new_v4());
        let mut model = UserModel::default();
        model.id = row_id;
        model.insert_date = Utc::now().timestamp() as i32;
        model.custom_id = Some(custom_id);
        diesel::insert_into(user::table).values(&vec![model]).execute(&mut *self.database)?;

        Ok(row_id)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Ok;

    use crate::{create_database, create_connection, PooledConnection};

    use super::{AuthStore, AuthStoreTrait};
    fn db_conection() -> anyhow::Result<PooledConnection> {
        let mut connection = create_connection(":memory:")?.get()?;
        create_database(&mut connection)?;
        Ok(connection)
    }

    /* email unit tests */
    #[test]
    fn create_user_via_email() -> anyhow::Result<()> {
        let connection = db_conection()?;

        let mut store = AuthStore::new(connection);
        store.create_user_via_email("erhanbaris@gmail.com", "erhan")?;
        Ok(())
    }

    #[test]
    fn login_via_email() -> anyhow::Result<()> {
        let connection = db_conection()?;

        let mut store = AuthStore::new(connection);
        let created_user_id = store.create_user_via_email("erhanbaris@gmail.com", "erhan")?;
        let (logged_user_id, name, password) = store.user_login_via_email("erhanbaris@gmail.com")?.unwrap();

        assert_eq!(created_user_id, logged_user_id);
        assert!(name.is_none());
        assert_eq!(password.as_str(), "erhan");

        Ok(())
    }

    #[test]
    fn failed_login_via_email() -> anyhow::Result<()> {
        let connection = db_conection()?;

        let mut store = AuthStore::new(connection);
        assert!(store.user_login_via_email("erhanbaris@gmail.com")?.is_none());

        Ok(())
    }

    /* device id unit tests */
    #[test]
    fn create_user_via_device_id() -> anyhow::Result<()> {
        let connection = db_conection()?;

        let mut store = AuthStore::new(connection);
        store.create_user_via_device_id("1234567890")?;
        Ok(())
    }

    #[test]
    fn login_via_device_id() -> anyhow::Result<()> {
        let connection = db_conection()?;

        let mut store = AuthStore::new(connection);

        let created_user_id = store.create_user_via_device_id("1234567890")?;
        let (logged_user_id, name, email) = store.user_login_via_device_id("1234567890")?.unwrap();

        assert_eq!(created_user_id, logged_user_id);
        assert!(name.is_none());
        assert!(email.is_none());

        Ok(())
    }

    #[test]
    fn failed_login_via_device_id() -> anyhow::Result<()> {
        let connection = db_conection()?;

        let mut store = AuthStore::new(connection);
        assert!(store.user_login_via_device_id("1234567890")?.is_none());
        Ok(())
    }

    /* custom id unit tests */
    #[test]
    fn create_user_via_custom_id() -> anyhow::Result<()> {
        let connection = db_conection()?;

        let mut store = AuthStore::new(connection);
        store.create_user_via_custom_id("1234567890")?;
        Ok(())
    }

    #[test]
    fn login_via_custom_id() -> anyhow::Result<()> {
        let connection = db_conection()?;

        let mut store = AuthStore::new(connection);

        let created_user_id = store.create_user_via_custom_id("1234567890")?;
        let (logged_user_id, name, email) = store.user_login_via_custom_id("1234567890")?.unwrap();

        assert_eq!(created_user_id, logged_user_id);
        assert!(name.is_none());
        assert!(email.is_none());

        Ok(())
    }

    #[test]
    fn failed_login_via_custom_id() -> anyhow::Result<()> {
        let connection = db_conection()?;

        let mut store = AuthStore::new(connection);
        assert!(store.user_login_via_custom_id("1234567890")?.is_none());
        Ok(())
    }
}