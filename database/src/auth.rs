use diesel::*;
use general::model::{UserType, UserId};
use general::password::Password;
use std::borrow::Borrow;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::SqliteStore;
use crate::model::LoginInfo;
use crate::{PooledConnection, schema::user, model::UserInsert};

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
        let result = user::table
            .filter(user::email.eq(email))
            .select((user::id, user::name, user::password, user::user_type))
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
        let result = user::table
            .filter(user::device_id.eq(device_id))
            .select((user::id, user::name, user::email, user::user_type))
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
        let result = user::table
            .filter(user::custom_id.eq(custom_id))
            .select((user::id, user::name, user::email, user::user_type))
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
        diesel::update(user::table.filter(user::id.eq(user_id.borrow())))
            .set(user::last_login_date.eq(SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default())).execute(connection)?;
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
        diesel::insert_into(user::table).values(&vec![model]).execute(connection)?;

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
        
        diesel::insert_into(user::table).values(&vec![model]).execute(connection)?;

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
        
        diesel::insert_into(user::table).values(&vec![model]).execute(connection)?;

        Ok(user_id)
    }
}
