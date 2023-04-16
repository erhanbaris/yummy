use diesel::*;
use yummy_model::password::Password;
use yummy_model::{UserId, UserType};
use yummy_model::user::UserInsert;
use yummy_model::user::LoginInfo;
use yummy_model::schema::user as user_schema;
use std::borrow::Borrow;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::SqliteStore;
use crate::PooledConnection;

pub trait AuthStoreTrait: Sized {
    fn user_login_via_email(connection: &mut PooledConnection, email: &str) -> anyhow::Result<Option<LoginInfo>>;
    fn user_login_via_device_id(connection: &mut PooledConnection, device_id: &str) -> anyhow::Result<Option<LoginInfo>>;
    fn user_login_via_custom_id(connection: &mut PooledConnection, custom_id: &str) -> anyhow::Result<Option<LoginInfo>>;

    fn update_last_login(connection: &mut PooledConnection, user_id: &UserId) -> anyhow::Result<()>;

    fn create_user_via_email(connection: &mut PooledConnection, email: &str, password: &Password) -> anyhow::Result<UserId>;
    fn create_user_via_device_id(connection: &mut PooledConnection, device_id: &str) -> anyhow::Result<UserId>;
    fn create_user_via_custom_id(connection: &mut PooledConnection, custom_id: &str) -> anyhow::Result<UserId>;
}

impl AuthStoreTrait for SqliteStore {
    #[tracing::instrument(name="User login via email", skip(connection))]
    fn user_login_via_email(connection: &mut PooledConnection, email: &str) -> anyhow::Result<Option<LoginInfo>> {
        let result = user_schema::table
            .filter(user_schema::email.eq(email))
            .select((user_schema::id, user_schema::name, user_schema::password, user_schema::user_type))
            .first::<(UserId, Option<String>, Option<String>, i32)>(connection)
            .optional()?
            .map(|(id, name, password, user_type)| (id, name, password.unwrap_or_default(), user_type.into()));

        match result {
            Some((user_id, name, password, user_type)) => Ok(Some(LoginInfo {
                user_id,
                name,
                email: None,
                password: Some(password),
                user_type
            })),
            None => Ok(None)
        }
    }

    #[tracing::instrument(name="User login via device id", skip(connection))]
    fn user_login_via_device_id(connection: &mut PooledConnection, device_id: &str) -> anyhow::Result<Option<LoginInfo>> {
        let result = user_schema::table
            .filter(user_schema::device_id.eq(device_id))
            .select((user_schema::id, user_schema::name, user_schema::email, user_schema::user_type))
            .first::<(UserId, Option<String>, Option<String>, i32)>(connection)
            .optional()?;

        match result {
            Some((user_id, name, email, user_type)) => {
                Self::update_last_login(connection, &user_id)?;
                Ok(Some(LoginInfo {
                    user_id,
                    name,
                    password: None,
                    email,
                    user_type: user_type.into()
                }))
            },
            None => Ok(None)
        }
    }

    #[tracing::instrument(name="User login via custom id", skip(connection))]
    fn user_login_via_custom_id(connection: &mut PooledConnection, custom_id: &str) -> anyhow::Result<Option<LoginInfo>> {
        let result = user_schema::table
            .filter(user_schema::custom_id.eq(custom_id))
            .select((user_schema::id, user_schema::name, user_schema::email, user_schema::user_type))
            .first::<(UserId, Option<String>, Option<String>, i32)>(connection)
            .optional()?;

        match result {
            Some((user_id, name, email, user_type)) => {
                Self::update_last_login(connection, &user_id)?;
                Ok(Some(LoginInfo {
                    user_id,
                    name,
                    password: None,
                    email,
                    user_type: user_type.into()
                }))
            },
            None => Ok(None)
        }
    }

    #[tracing::instrument(name="Update last login", skip(connection))]
    fn update_last_login(connection: &mut PooledConnection, user_id: &UserId) -> anyhow::Result<()> {
        diesel::update(user_schema::table.filter(user_schema::id.eq(user_id.borrow())))
            .set(user_schema::last_login_date.eq(SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default())).execute(connection)?;
        Ok(())
    }

    #[tracing::instrument(name="User create via email", skip(connection))]
    fn create_user_via_email(connection: &mut PooledConnection, email: &str, password: &Password) -> anyhow::Result<UserId> {
        
        let user_id = UserId::default();
        let insert_date = SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default();
        let model = UserInsert {
            id: &user_id,
            insert_date,
            last_login_date: insert_date,
            password: Some(password.get()),
            email: Some(email),
            user_type: UserType::default().into(),
            custom_id: None,
            device_id: None,
            name: None
        };
        diesel::insert_into(user_schema::table).values(&vec![model]).execute(connection)?;

        Ok(user_id)
    }

    #[tracing::instrument(name="User create via device id", skip(connection))]
    fn create_user_via_device_id(connection: &mut PooledConnection, device_id: &str) -> anyhow::Result<UserId> {
        
        let user_id = UserId::default();
        let insert_date = SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default();
        let model = UserInsert {
            id: &user_id,
            insert_date,
            last_login_date: insert_date,
            device_id: Some(device_id),
            user_type: UserType::default().into(),
            custom_id: None,
            email: None,
            name: None,
            password: None
        };
        
        diesel::insert_into(user_schema::table).values(&vec![model]).execute(connection)?;

        Ok(user_id)
    }

    #[tracing::instrument(name="User create via custom id", skip(connection))]
    fn create_user_via_custom_id(connection: &mut PooledConnection, custom_id: &str) -> anyhow::Result<UserId> {

        let user_id = UserId::default();
        let insert_date = SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default();
        let model = UserInsert {
            id: &user_id,
            insert_date,
            last_login_date: insert_date,
            user_type: UserType::default().into(),
            custom_id: Some(custom_id),
            device_id: None,
            email: None,
            name: None,
            password: None
        };
        
        diesel::insert_into(user_schema::table).values(&vec![model]).execute(connection)?;

        Ok(user_id)
    }
}
