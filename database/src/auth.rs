use chrono::Utc;
use diesel::*;
use uuid::Uuid;

use crate::SqliteStore;
use crate::{PooledConnection, schema::user, RowId, model::UserInsert};

pub trait AuthStoreTrait: Sized {
    fn user_login_via_email(connection: &mut PooledConnection, email: &str) -> anyhow::Result<Option<(RowId, Option<String>, String)>>;
    fn user_login_via_device_id(connection: &mut PooledConnection, device_id: &str) -> anyhow::Result<Option<(RowId, Option<String>, Option<String>)>>;
    fn user_login_via_custom_id(connection: &mut PooledConnection, custom_id: &str) -> anyhow::Result<Option<(RowId, Option<String>, Option<String>)>>;

    fn create_user_via_email(connection: &mut PooledConnection, email: &str, password: &str) -> anyhow::Result<RowId>;
    fn create_user_via_device_id(connection: &mut PooledConnection, device_id: &str) -> anyhow::Result<RowId>;
    fn create_user_via_custom_id(connection: &mut PooledConnection, custom_id: &str) -> anyhow::Result<RowId>;
}

impl AuthStoreTrait for SqliteStore {
    #[tracing::instrument(name="User login via email", skip(connection))]
    fn user_login_via_email(connection: &mut PooledConnection, email: &str) -> anyhow::Result<Option<(RowId, Option<String>, String)>> {
        let result = user::table
            .filter(user::email.eq(email))
            .select((user::id, user::name, user::password))
            .first::<(RowId, Option<String>, Option<String>)>(connection)
            .optional()?
            .map(|(id, name, password)| (id, name, password.unwrap_or_default()));

        Ok(result)
    }

    #[tracing::instrument(name="User login via device id", skip(connection))]
    fn user_login_via_device_id(connection: &mut PooledConnection, device_id: &str) -> anyhow::Result<Option<(RowId, Option<String>, Option<String>)>> {
        let result = user::table
            .filter(user::device_id.eq(device_id))
            .select((user::id, user::name, user::email))
            .first::<(RowId, Option<String>, Option<String>)>(connection)
            .optional()?;

        Ok(result)
    }

    #[tracing::instrument(name="User login via custom id", skip(connection))]
    fn user_login_via_custom_id(connection: &mut PooledConnection, custom_id: &str) -> anyhow::Result<Option<(RowId, Option<String>, Option<String>)>> {
        let result = user::table
            .filter(user::custom_id.eq(custom_id))
            .select((user::id, user::name, user::email))
            .first::<(RowId, Option<String>, Option<String>)>(connection)
            .optional()?;

        Ok(result)
    }

    #[tracing::instrument(name="User create via email", skip(connection))]
    fn create_user_via_email(connection: &mut PooledConnection, email: &str, password: &str) -> anyhow::Result<RowId> {
        
        let row_id = RowId(Uuid::new_v4());
        let mut model = UserInsert::default();
        model.id = row_id;
        model.insert_date = Utc::now().timestamp() as i32;
        model.password = Some(password);
        model.email = Some(email);
        diesel::insert_into(user::table).values(&vec![model]).execute(connection)?;

        Ok(row_id)
    }

    #[tracing::instrument(name="User create via device id", skip(connection))]
    fn create_user_via_device_id(connection: &mut PooledConnection, device_id: &str) -> anyhow::Result<RowId> {
        let row_id = RowId(Uuid::new_v4());
        let mut model = UserInsert::default();
        model.id = row_id;
        model.insert_date = Utc::now().timestamp() as i32;
        model.device_id = Some(device_id);
        diesel::insert_into(user::table).values(&vec![model]).execute(connection)?;

        Ok(row_id)
    }

    #[tracing::instrument(name="User create via custom id", skip(connection))]
    fn create_user_via_custom_id(connection: &mut PooledConnection, custom_id: &str) -> anyhow::Result<RowId> {
        let row_id = RowId(Uuid::new_v4());
        let mut model = UserInsert::default();
        model.id = row_id;
        model.insert_date = Utc::now().timestamp() as i32;
        model.custom_id = Some(custom_id);
        diesel::insert_into(user::table).values(&vec![model]).execute(connection)?;

        Ok(row_id)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Ok;

    use crate::{create_database, create_connection, PooledConnection};

    use crate::SqliteStore;
    use super::*;
    fn db_conection() -> anyhow::Result<PooledConnection> {
        let mut connection = create_connection(":memory:")?.get()?;
        create_database(&mut connection)?;
        Ok(connection)
    }

    /* email unit tests */
    #[test]
    fn create_user_via_email() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        SqliteStore::create_user_via_email(&mut connection, "erhanbaris@gmail.com", "erhan")?;
        Ok(())
    }

    #[test]
    fn login_via_email() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let created_user_id = SqliteStore::create_user_via_email(&mut connection, "erhanbaris@gmail.com", "erhan")?;
        let (logged_user_id, name, password) = SqliteStore::user_login_via_email(&mut connection, "erhanbaris@gmail.com")?.unwrap();

        assert_eq!(created_user_id, logged_user_id);
        assert!(name.is_none());
        assert_eq!(password.as_str(), "erhan");

        Ok(())
    }

    #[test]
    fn failed_login_via_email() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert!(SqliteStore::user_login_via_email(&mut connection, "erhanbaris@gmail.com")?.is_none());

        Ok(())
    }

    /* device id unit tests */
    #[test]
    fn create_user_via_device_id() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        SqliteStore::create_user_via_device_id(&mut connection, "1234567890")?;
        Ok(())
    }

    #[test]
    fn login_via_device_id() -> anyhow::Result<()> {
        let mut connection = db_conection()?;


        let created_user_id = SqliteStore::create_user_via_device_id(&mut connection, "1234567890")?;
        let (logged_user_id, name, email) = SqliteStore::user_login_via_device_id(&mut connection, "1234567890")?.unwrap();

        assert_eq!(created_user_id, logged_user_id);
        assert!(name.is_none());
        assert!(email.is_none());

        Ok(())
    }

    #[test]
    fn failed_login_via_device_id() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert!(SqliteStore::user_login_via_device_id(&mut connection, "1234567890")?.is_none());
        Ok(())
    }

    /* custom id unit tests */
    #[test]
    fn create_user_via_custom_id() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        SqliteStore::create_user_via_custom_id(&mut connection, "1234567890")?;
        Ok(())
    }

    #[test]
    fn login_via_custom_id() -> anyhow::Result<()> {
        let mut connection = db_conection()?;


        let created_user_id = SqliteStore::create_user_via_custom_id(&mut connection, "1234567890")?;
        let (logged_user_id, name, email) = SqliteStore::user_login_via_custom_id(&mut connection, "1234567890")?.unwrap();

        assert_eq!(created_user_id, logged_user_id);
        assert!(name.is_none());
        assert!(email.is_none());

        Ok(())
    }

    #[test]
    fn failed_login_via_custom_id() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert!(SqliteStore::user_login_via_custom_id(&mut connection, "1234567890")?.is_none());
        Ok(())
    }
}