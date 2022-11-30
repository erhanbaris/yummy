use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use diesel::QueryDsl;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use diesel::result::OptionalExtension;
use general::meta::MetaType;
use general::meta::Visibility;
use uuid::Uuid;

use crate::SqliteStore;
use crate::model::PrivateUserModel;
use crate::model::PublicUserModel;
use crate::model::UserMetaInsert;
use crate::model::UserMetaModel;
use crate::model::UserUpdate;
use crate::schema::user_meta;
use crate::{PooledConnection, RowId, schema::user};

pub trait UserStoreTrait: Sized {
    fn update_user(connection: &mut PooledConnection, user_id: RowId, update_request: UserUpdate) -> anyhow::Result<usize>;
    fn get_user_meta(connection: &mut PooledConnection, user_id: RowId) -> anyhow::Result<Vec<(RowId, String, MetaType)>>;
    fn remove_user_metas(connection: &mut PooledConnection, meta_ids: Vec<RowId>) -> anyhow::Result<()>;
    fn insert_user_metas(connection: &mut PooledConnection, user_id: RowId, metas: Vec<(String, MetaType)>) -> anyhow::Result<()>;
    fn get_private_user_info(connection: &mut PooledConnection, user_id: RowId) -> anyhow::Result<Option<PrivateUserModel>>;
    fn get_public_user_info(connection: &mut PooledConnection, user_id: RowId) -> anyhow::Result<Option<PublicUserModel>>;
}

impl UserStoreTrait for SqliteStore {
    #[tracing::instrument(name="Update user", skip(connection))]
    fn update_user<'a>(connection: &mut PooledConnection, user_id: RowId, update_request: UserUpdate) -> anyhow::Result<usize> {
        Ok(diesel::update(user::table.filter(user::id.eq(user_id)))
            .set(&update_request).execute(connection)?)
    }

    #[tracing::instrument(name="Get user", skip(connection))]
    fn get_private_user_info<'a>(connection: &mut PooledConnection, user_id: RowId) -> anyhow::Result<Option<PrivateUserModel>> {
        Ok(user::table
            .select((user::id, user::name, user::email, user::device_id, user::custom_id, user::insert_date, user::last_login_date))
            .filter(user::id.eq(user_id))
            .get_result::<PrivateUserModel>(connection)
            .optional()?)
    }

    #[tracing::instrument(name="Get user", skip(connection))]
    fn get_public_user_info<'a>(connection: &mut PooledConnection, user_id: RowId) -> anyhow::Result<Option<PublicUserModel>> {
        Ok(user::table
            .select((user::id, user::name, user::last_login_date))
            .filter(user::id.eq(user_id))
            .get_result::<PublicUserModel>(connection)
            .optional()?)
    }

    #[tracing::instrument(name="Get user meta", skip(connection))]
    fn get_user_meta(connection: &mut PooledConnection, user_id: RowId) -> anyhow::Result<Vec<(RowId, String, MetaType)>> {
        let records: Vec<UserMetaModel> = user_meta::table
            .select((user_meta::id, user_meta::key, user_meta::value, user_meta::meta_type, user_meta::access))
            .filter(user_meta::user_id.eq(user_id))
            .load::<UserMetaModel>(connection)?;

            let records = records.into_iter().map(|record| {
                let UserMetaModel { id, key, value, meta_type, access } = record;

                let access = match access {
                    0 => Visibility::Anonymous,
                    1 => Visibility::User,
                    2 => Visibility::Friend,
                    3 => Visibility::Mod,
                    4 => Visibility::Admin,
                    5 => Visibility::System,
                    _ => Visibility::Anonymous
                };

                let meta = match meta_type {
                    1 => MetaType::Integer(value.parse::<i64>().unwrap_or_default(), access),
                    2 => MetaType::Float(value.parse::<f64>().unwrap_or_default(), access),
                    3 => MetaType::String(value, access),
                    4 => MetaType::Bool(value.parse::<bool>().unwrap_or_default(), access),
                    _ => MetaType::String("".to_string(), access),
                };

                (id, key, meta)
            }).collect();
            
        Ok(records)
    }

    #[tracing::instrument(name="Remove metas", skip(connection))]
    fn remove_user_metas(connection: &mut PooledConnection, ids: Vec<RowId>) -> anyhow::Result<()> {
        diesel::delete(user_meta::table.filter(user_meta::id.eq_any(ids))).execute(connection)?;
        Ok(())
    }

    #[tracing::instrument(name="Insert metas", skip(connection))]
    fn insert_user_metas(connection: &mut PooledConnection, user_id: RowId, metas: Vec<(String, MetaType)>) -> anyhow::Result<()> {
        let insert_date = SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default();
        let mut inserts = Vec::new();

        for (key, meta) in metas.into_iter() {
            let id = RowId(Uuid::new_v4());
            let (value, access, meta_type) = match meta {
                MetaType::Null => continue,
                MetaType::Integer(value, access) => (value.to_string(), access, 1),
                MetaType::Float(value, access) => (value.to_string(), access, 2),
                MetaType::String(value, access) => (value, access, 3),
                MetaType::Bool(value, access) => (value.to_string(), access, 4),
            };

            let insert = UserMetaInsert {
                id,
                user_id: &user_id,
                key,
                value,
                access: match access {
                    Visibility::Anonymous => 0,
                    Visibility::User => 1,
                    Visibility::Friend => 2,
                    Visibility::Mod => 3,
                    Visibility::Admin => 4,
                    Visibility::System => 5,
                },
                meta_type,
                insert_date
            };

            inserts.push(insert);
        }
        diesel::insert_into(user_meta::table).values(&inserts).execute(connection)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;

    use anyhow::Ok;
    use uuid::Uuid;
    use crate::SqliteStore;
    use crate::{create_database, create_connection, PooledConnection};
    use crate::auth::*;

    use super::*;

    fn db_conection() -> anyhow::Result<PooledConnection> {
        let mut db_location = temp_dir();
        db_location.push(format!("{}.db", Uuid::new_v4()));

        let mut connection = create_connection(db_location.to_str().unwrap())?.get()?;
        create_database(&mut connection)?;
        Ok(connection)
    }

    /* get user tests */
    #[test]
    fn fail_get_public_user_info_1() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert!(SqliteStore::get_public_user_info(&mut connection, RowId(uuid::Uuid::nil()))?.is_none());
        Ok(())
    }

    #[test]
    fn fail_get_public_user_info_2() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert!(SqliteStore::get_public_user_info(&mut connection, RowId(uuid::Uuid::new_v4()))?.is_none());
        Ok(())
    }

    #[test]
    fn get_public_user_info_1() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let user_id = SqliteStore::create_user_via_email(&mut connection, "erhanbaris@gmail.com", "erhan")?;
        let user = SqliteStore::get_public_user_info(&mut connection, user_id)?.unwrap();
        assert_eq!(user.id, user_id);
        assert_eq!(user.last_login_date, 0);
        assert!(user.name.is_none());

        Ok(())
    }

    #[test]
    fn get_public_user_info_2() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let user_id = SqliteStore::create_user_via_device_id(&mut connection, "123456789")?;
        let user = SqliteStore::get_public_user_info(&mut connection, user_id)?.unwrap();
        assert_eq!(user.id, user_id);
        assert_eq!(user.last_login_date, 0);
        assert!(user.name.is_none());

        Ok(())
    }

    #[test]
    fn get_public_user_info_3() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let user_id = SqliteStore::create_user_via_custom_id(&mut connection, "123456789")?;
        let user = SqliteStore::get_public_user_info(&mut connection, user_id)?.unwrap();
        assert_eq!(user.id, user_id);
        assert_eq!(user.last_login_date, 0);
        assert!(user.name.is_none());

        Ok(())
    }

    #[test]
    fn fail_get_private_user_info_1() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert!(SqliteStore::get_private_user_info(&mut connection, RowId(uuid::Uuid::nil()))?.is_none());
        Ok(())
    }

    #[test]
    fn fail_get_private_user_info_2() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert!(SqliteStore::get_private_user_info(&mut connection, RowId(uuid::Uuid::new_v4()))?.is_none());
        Ok(())
    }

    #[test]
    fn get_private_user_info_1() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let user_id = SqliteStore::create_user_via_email(&mut connection, "erhanbaris@gmail.com", "erhan")?;
        let user = SqliteStore::get_private_user_info(&mut connection, user_id)?.unwrap();
        assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));
        assert!(user.custom_id.is_none());
        assert!(user.device_id.is_none());
        assert!(user.name.is_none());

        Ok(())
    }

    #[test]
    fn get_private_user_info_2() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let user_id = SqliteStore::create_user_via_device_id(&mut connection, "123456789")?;
        let user = SqliteStore::get_private_user_info(&mut connection, user_id)?.unwrap();
        assert_eq!(user.device_id, Some("123456789".to_string()));
        assert!(user.email.is_none());
        assert!(user.custom_id.is_none());
        assert!(user.name.is_none());

        Ok(())
    }

    #[test]
    fn get_private_user_info_3() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let user_id = SqliteStore::create_user_via_custom_id(&mut connection, "123456789")?;
        let user = SqliteStore::get_private_user_info(&mut connection, user_id)?.unwrap();
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