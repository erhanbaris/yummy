use diesel::QueryDsl;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use diesel::result::OptionalExtension;

use crate::SqliteStore;
use crate::model::PrivateUserModel;
use crate::model::UserUpdate;
use crate::{PooledConnection, RowId, schema::user};

pub trait UserStoreTrait: Sized {
    fn update_user(connection: &mut PooledConnection, user_id: RowId, update_request: UserUpdate) -> anyhow::Result<usize>;
    fn get_user(connection: &mut PooledConnection, user_id: RowId) -> anyhow::Result<Option<PrivateUserModel>>;
}

impl UserStoreTrait for SqliteStore {
    #[tracing::instrument(name="Update user", skip(connection))]
    fn update_user<'a>(connection: &mut PooledConnection, user_id: RowId, update_request: UserUpdate) -> anyhow::Result<usize> {
        Ok(diesel::update(user::table.filter(user::id.eq(user_id)))
            .set(&update_request).execute(connection)?)
    }

    #[tracing::instrument(name="Get user", skip(connection))]
    fn get_user<'a>(connection: &mut PooledConnection, user_id: RowId) -> anyhow::Result<Option<PrivateUserModel>> {
        Ok(user::table
            .select((user::id, user::name, user::email, user::device_id, user::custom_id, user::insert_date, user::last_login_date))
            .filter(user::id.eq(user_id))
            .get_result::<PrivateUserModel>(connection)
            .optional()?)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Ok;
    use crate::SqliteStore;
    use crate::{create_database, create_connection, PooledConnection};
    use crate::auth::*;

    use super::*;

    fn db_conection() -> anyhow::Result<PooledConnection> {
        let mut connection = create_connection(":memory:")?.get()?;
        create_database(&mut connection)?;
        Ok(connection)
    }

    /* get user tests */
    #[test]
    fn fail_get_user_1() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert!(SqliteStore::get_user(&mut connection, RowId(uuid::Uuid::nil()))?.is_none());
        Ok(())
    }

    #[test]
    fn fail_get_user_2() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert!(SqliteStore::get_user(&mut connection, RowId(uuid::Uuid::new_v4()))?.is_none());
        Ok(())
    }

    #[test]
    fn get_user_1() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let user_id = SqliteStore::create_user_via_email(&mut connection, "erhanbaris@gmail.com", "erhan")?;
        let user = SqliteStore::get_user(&mut connection, user_id)?.unwrap();
        assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));
        assert!(user.custom_id.is_none());
        assert!(user.device_id.is_none());
        assert!(user.name.is_none());

        Ok(())
    }

    #[test]
    fn get_user_2() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let user_id = SqliteStore::create_user_via_device_id(&mut connection, "123456789")?;
        let user = SqliteStore::get_user(&mut connection, user_id)?.unwrap();
        assert_eq!(user.device_id, Some("123456789".to_string()));
        assert!(user.email.is_none());
        assert!(user.custom_id.is_none());
        assert!(user.name.is_none());

        Ok(())
    }

    #[test]
    fn get_user_3() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let user_id = SqliteStore::create_user_via_custom_id(&mut connection, "123456789")?;
        let user = SqliteStore::get_user(&mut connection, user_id)?.unwrap();
        assert_eq!(user.custom_id, Some("123456789".to_string()));
        assert!(user.email.is_none());
        assert!(user.device_id.is_none());
        assert!(user.name.is_none());

        Ok(())
    }

    /* update user tests */
    #[test]
    fn fail_update_user_1() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert!(SqliteStore::update_user(&mut connection, RowId(uuid::Uuid::new_v4()), UserUpdate::default()).is_err());
        Ok(())
    }

    #[test]
    fn fail_update_user_2() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert!(SqliteStore::update_user(&mut connection, RowId(uuid::Uuid::nil()), UserUpdate::default()).is_err());
        Ok(())
    }
    #[test]
    fn fail_update_user_3() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert_eq!(SqliteStore::update_user(&mut connection, RowId(uuid::Uuid::new_v4()), UserUpdate {
            name: Some(Some("123456".to_string())),
            ..Default::default()
        })?, 0);
        Ok(())
    }

    #[test]
    fn fail_update_user_4() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert_eq!(SqliteStore::update_user(&mut connection, RowId(uuid::Uuid::nil()), UserUpdate {
            name: Some(Some("123456".to_string())),
            ..Default::default()
        })?, 0);
        Ok(())
    }

    #[test]
    fn update_user() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let user_id = SqliteStore::create_user_via_custom_id(&mut connection, "123456789")?;
        assert_eq!(SqliteStore::update_user(&mut connection, user_id, UserUpdate {
            name: Some(Some("123456".to_string())),
            ..Default::default()
        })?, 1);
        Ok(())
    }

}