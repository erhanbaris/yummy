use std::collections::HashMap;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use diesel::QueryDsl;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use diesel::result::OptionalExtension;
use general::meta::MetaType;
use general::meta::MetaAccess;
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
    fn get_user_meta(connection: &mut PooledConnection, user_id: RowId, filter: MetaAccess) -> anyhow::Result<Vec<(RowId, String, MetaType)>>;
    fn remove_user_metas(connection: &mut PooledConnection, meta_ids: Vec<RowId>) -> anyhow::Result<()>;
    fn insert_user_metas(connection: &mut PooledConnection, user_id: RowId, metas: Vec<(String, MetaType)>) -> anyhow::Result<()>;
    fn get_my_information(connection: &mut PooledConnection, user_id: RowId) -> anyhow::Result<Option<PrivateUserModel>>;
    fn get_public_user_info(connection: &mut PooledConnection, user_id: RowId) -> anyhow::Result<Option<PublicUserModel>>;
}

impl UserStoreTrait for SqliteStore {
    #[tracing::instrument(name="Update user", skip(connection))]
    fn update_user<'a>(connection: &mut PooledConnection, user_id: RowId, update_request: UserUpdate) -> anyhow::Result<usize> {
        Ok(diesel::update(user::table.filter(user::id.eq(user_id)))
            .set(&update_request).execute(connection)?)
    }

    #[tracing::instrument(name="Get user", skip(connection))]
    fn get_my_information<'a>(connection: &mut PooledConnection, user_id: RowId) -> anyhow::Result<Option<PrivateUserModel>> {
        let result = user::table
            .select((user::id, user::name, user::email, user::device_id, user::custom_id, user::insert_date, user::last_login_date))
            .filter(user::id.eq(user_id))
            .get_result::<(RowId, Option<String>, Option<String>, Option<String>, Option<String>, i32, i32)>(connection)
            .optional()?;

        match result {
            Some((id, name, email, device_id, custom_id, insert_date, last_login_date)) => {
                let meta: HashMap<_, _> = Self::get_user_meta(connection, user_id, MetaAccess::Me)?.into_iter().map(|(_, key, value)| (key, value)).collect();
                let meta = match meta.is_empty() {
                    true => None,
                    false => Some(meta)
                };
                Ok(Some(PrivateUserModel { id, name, email, device_id, custom_id, meta, insert_date, last_login_date }))
            },
            None => Ok(None)
        }
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
    fn get_user_meta(connection: &mut PooledConnection, user_id: RowId, filter: MetaAccess) -> anyhow::Result<Vec<(RowId, String, MetaType)>> {
        let records: Vec<UserMetaModel> = user_meta::table
            .select((user_meta::id, user_meta::key, user_meta::value, user_meta::meta_type, user_meta::access))
            .filter(user_meta::user_id.eq(user_id))
            .filter(user_meta::access.le(i32::from(filter)))
            .load::<UserMetaModel>(connection)?;

            let records = records.into_iter().map(|record| {
                let UserMetaModel { id, key, value, meta_type, access } = record;

                let meta = match meta_type {
                    1 => MetaType::Integer(value.parse::<i64>().unwrap_or_default(), access.into()),
                    2 => MetaType::Float(value.parse::<f64>().unwrap_or_default(), access.into()),
                    3 => MetaType::String(value, access.into()),
                    4 => MetaType::Bool(value.parse::<bool>().unwrap_or_default(), access.into()),
                    _ => MetaType::String("".to_string(), access.into()),
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
                access: access.into(),
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
    fn fail_get_my_information_1() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert!(SqliteStore::get_my_information(&mut connection, RowId(uuid::Uuid::nil()))?.is_none());
        Ok(())
    }

    #[test]
    fn fail_get_my_information_2() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert!(SqliteStore::get_my_information(&mut connection, RowId(uuid::Uuid::new_v4()))?.is_none());
        Ok(())
    }

    #[test]
    fn get_my_information_1() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let user_id = SqliteStore::create_user_via_email(&mut connection, "erhanbaris@gmail.com", "erhan")?;
        let user = SqliteStore::get_my_information(&mut connection, user_id)?.unwrap();
        assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));
        assert!(user.custom_id.is_none());
        assert!(user.device_id.is_none());
        assert!(user.name.is_none());

        Ok(())
    }

    #[test]
    fn get_my_information_2() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let user_id = SqliteStore::create_user_via_device_id(&mut connection, "123456789")?;
        let user = SqliteStore::get_my_information(&mut connection, user_id)?.unwrap();
        assert_eq!(user.device_id, Some("123456789".to_string()));
        assert!(user.email.is_none());
        assert!(user.custom_id.is_none());
        assert!(user.name.is_none());

        Ok(())
    }

    #[test]
    fn get_my_information_3() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let user_id = SqliteStore::create_user_via_custom_id(&mut connection, "123456789")?;
        let user = SqliteStore::get_my_information(&mut connection, user_id)?.unwrap();
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

    #[test]
    fn meta() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let user_id = SqliteStore::create_user_via_custom_id(&mut connection, "123456789")?;
        assert_eq!(SqliteStore::get_user_meta(&mut connection, user_id, MetaAccess::System)?.len(), 0);

        // New meta
        SqliteStore::insert_user_metas(&mut connection, user_id, vec![("gender".to_string(), MetaType::String("male".to_string(), MetaAccess::Friend))])?;

        assert_eq!(SqliteStore::get_user_meta(&mut connection, user_id, MetaAccess::Friend)?.len(), 1);
        assert_eq!(SqliteStore::get_user_meta(&mut connection, user_id, MetaAccess::Anonymous)?.len(), 0);

        let meta = SqliteStore::get_user_meta(&mut connection, user_id, MetaAccess::System)?;
        assert_eq!(meta.len(), 1);

        // Remove meta
        SqliteStore::remove_user_metas(&mut connection, vec![meta[0].0])?;
        assert_eq!(SqliteStore::get_user_meta(&mut connection, user_id, MetaAccess::Friend)?.len(), 0);
        assert_eq!(SqliteStore::get_user_meta(&mut connection, user_id, MetaAccess::Anonymous)?.len(), 0);
        assert_eq!(SqliteStore::get_user_meta(&mut connection, user_id, MetaAccess::System)?.len(), 0);

        SqliteStore::insert_user_metas(&mut connection, user_id, vec![
            ("location".to_string(), MetaType::String("copenhagen".to_string(), MetaAccess::Anonymous)),
            ("score".to_string(), MetaType::Integer(123, MetaAccess::Friend))])?;

        assert_eq!(SqliteStore::get_user_meta(&mut connection, user_id, MetaAccess::Friend)?.len(), 2);
        assert_eq!(SqliteStore::get_user_meta(&mut connection, user_id, MetaAccess::System)?.len(), 2);

        // Filter with anonymous
        let meta = SqliteStore::get_user_meta(&mut connection, user_id, MetaAccess::Anonymous)?;
        assert_eq!(meta.len(), 1);
        assert_eq!(meta.into_iter().map(|(_, key, value)| (key, value)).collect::<Vec<(String, MetaType)>>(), vec![
            ("location".to_string(), MetaType::String("copenhagen".to_string(), MetaAccess::Anonymous))]);

        // Filter with system
        let meta = SqliteStore::get_user_meta(&mut connection, user_id, MetaAccess::System)?;
        assert_eq!(meta.len(), 2);
        assert_eq!(meta.into_iter().map(|(_, key, value)| (key, value)).collect::<Vec<(String, MetaType)>>(), vec![
            ("location".to_string(), MetaType::String("copenhagen".to_string(), MetaAccess::Anonymous)),
            ("score".to_string(), MetaType::Integer(123, MetaAccess::Friend))]);

        Ok(())
    }

}