use diesel::*;
use general::model::UserType;
use uuid::Uuid;
use std::borrow::Borrow;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::SqliteStore;
use crate::model::LoginInfo;
use crate::{PooledConnection, schema::user, RowId, model::UserInsert};

pub trait AuthStoreTrait: Sized {
    fn user_login_via_email(connection: &mut PooledConnection, email: &str) -> anyhow::Result<Option<LoginInfo>>;
    fn user_login_via_device_id(connection: &mut PooledConnection, device_id: &str) -> anyhow::Result<Option<LoginInfo>>;
    fn user_login_via_custom_id(connection: &mut PooledConnection, custom_id: &str) -> anyhow::Result<Option<LoginInfo>>;

    fn update_last_login<T: Borrow<RowId> + std::fmt::Debug>(connection: &mut PooledConnection, user_id: T) -> anyhow::Result<()>;

    fn create_user_via_email(connection: &mut PooledConnection, email: &str, password: &str) -> anyhow::Result<RowId>;
    fn create_user_via_device_id(connection: &mut PooledConnection, device_id: &str) -> anyhow::Result<RowId>;
    fn create_user_via_custom_id(connection: &mut PooledConnection, custom_id: &str) -> anyhow::Result<RowId>;
}

impl AuthStoreTrait for SqliteStore {
    #[tracing::instrument(name="User login via email", skip(connection))]
    fn user_login_via_email(connection: &mut PooledConnection, email: &str) -> anyhow::Result<Option<LoginInfo>> {
        let result = user::table
            .filter(user::email.eq(email))
            .select((user::id, user::name, user::password))
            .first::<(RowId, Option<String>, Option<String>)>(connection)
            .optional()?
            .map(|(id, name, password)| (id, name, password.unwrap_or_default()));

        match result {
            Some((user_id, name, password)) => Ok(Some(LoginInfo {
                user_id,
                name,
                email: None,
                password: Some(password)
            })),
            None => Ok(None)
        }
    }

    #[tracing::instrument(name="User login via device id", skip(connection))]
    fn user_login_via_device_id(connection: &mut PooledConnection, device_id: &str) -> anyhow::Result<Option<LoginInfo>> {
        let result = user::table
            .filter(user::device_id.eq(device_id))
            .select((user::id, user::name, user::email))
            .first::<(RowId, Option<String>, Option<String>)>(connection)
            .optional()?;

        match result {
            Some((user_id, name, email)) => {
                Self::update_last_login(connection, user_id)?;
                Ok(Some(LoginInfo {
                    user_id,
                    name,
                    password: None,
                    email
                }))
            },
            None => Ok(None)
        }
    }

    #[tracing::instrument(name="User login via custom id", skip(connection))]
    fn user_login_via_custom_id(connection: &mut PooledConnection, custom_id: &str) -> anyhow::Result<Option<LoginInfo>> {
        let result = user::table
            .filter(user::custom_id.eq(custom_id))
            .select((user::id, user::name, user::email))
            .first::<(RowId, Option<String>, Option<String>)>(connection)
            .optional()?;

        match result {
            Some((user_id, name, email)) => {
                Self::update_last_login(connection, user_id)?;
                Ok(Some(LoginInfo {
                    user_id,
                    name,
                    password: None,
                    email
                }))
            },
            None => Ok(None)
        }
    }

    #[tracing::instrument(name="Update last login", skip(connection))]
    fn update_last_login<T: Borrow<RowId> + std::fmt::Debug>(connection: &mut PooledConnection, user_id: T) -> anyhow::Result<()> {
        diesel::update(user::table.filter(user::id.eq(user_id.borrow())))
            .set(user::last_login_date.eq(SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default())).execute(connection)?;
        Ok(())
    }

    #[tracing::instrument(name="User create via email", skip(connection))]
    fn create_user_via_email(connection: &mut PooledConnection, email: &str, password: &str) -> anyhow::Result<RowId> {
        
        let row_id = RowId(Uuid::new_v4());
        let mut model = UserInsert::default();
        model.id = row_id;
        model.insert_date = SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default();
        model.last_login_date = model.insert_date;
        model.password = Some(password);
        model.email = Some(email);
        model.user_type = UserType::default().into();
        diesel::insert_into(user::table).values(&vec![model]).execute(connection)?;

        Ok(row_id)
    }

    #[tracing::instrument(name="User create via device id", skip(connection))]
    fn create_user_via_device_id(connection: &mut PooledConnection, device_id: &str) -> anyhow::Result<RowId> {
        let row_id = RowId(Uuid::new_v4());
        let mut model = UserInsert::default();
        model.id = row_id;
        model.insert_date = SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default();
        model.last_login_date = model.insert_date;
        model.device_id = Some(device_id);
        model.user_type = UserType::default().into();
        diesel::insert_into(user::table).values(&vec![model]).execute(connection)?;

        Ok(row_id)
    }

    #[tracing::instrument(name="User create via custom id", skip(connection))]
    fn create_user_via_custom_id(connection: &mut PooledConnection, custom_id: &str) -> anyhow::Result<RowId> {
        let row_id = RowId(Uuid::new_v4());
        let mut model = UserInsert::default();
        model.id = row_id;
        model.insert_date = SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default();
        model.last_login_date = model.insert_date;
        model.custom_id = Some(custom_id);
        model.user_type = UserType::default().into();
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
        let result = SqliteStore::user_login_via_email(&mut connection, "erhanbaris@gmail.com")?.unwrap();

        assert_eq!(created_user_id, result.user_id);
        assert!(result.name.is_none());
        assert_eq!(result.password.unwrap_or_default().as_str(), "erhan");

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
        let result = SqliteStore::user_login_via_device_id(&mut connection, "1234567890")?.unwrap();

        assert_eq!(created_user_id, result.user_id);
        assert!(result.name.is_none());
        assert!(result.email.is_none());

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
        let result = SqliteStore::user_login_via_custom_id(&mut connection, "1234567890")?.unwrap();

        assert_eq!(created_user_id, result.user_id);
        assert!(result.name.is_none());
        assert!(result.email.is_none());

        Ok(())
    }

    #[test]
    fn failed_login_via_custom_id() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert!(SqliteStore::user_login_via_custom_id(&mut connection, "1234567890")?.is_none());
        Ok(())
    }
}