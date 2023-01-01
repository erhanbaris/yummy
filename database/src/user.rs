use std::borrow::Borrow;
use std::collections::HashMap;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use diesel::QueryDsl;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use diesel::result::OptionalExtension;
use general::meta::MetaType;
use general::meta::UserMetaAccess;
use general::model::UserId;
use general::model::UserMetaId;
use general::model::UserType;

use crate::SqliteStore;
use crate::model::UserInformationModel;
use crate::model::UserMetaInsert;
use crate::model::UserMetaModel;
use crate::model::UserUpdate;
use crate::schema::user_meta;
use crate::{PooledConnection, schema::user};

pub trait UserStoreTrait: Sized {
    fn update_user(connection: &mut PooledConnection, user_id: &UserId, update_request: &UserUpdate) -> anyhow::Result<usize>;
    fn get_user_meta(connection: &mut PooledConnection, user_id: &UserId, filter: UserMetaAccess) -> anyhow::Result<Vec<(UserMetaId, String, MetaType<UserMetaAccess>)>>;
    fn remove_user_metas(connection: &mut PooledConnection, meta_ids: Vec<UserMetaId>) -> anyhow::Result<()>;
    fn insert_user_metas(connection: &mut PooledConnection, user_id: &UserId, metas: Vec<(String, MetaType<UserMetaAccess>)>) -> anyhow::Result<()>;
    fn get_user_information(connection: &mut PooledConnection, user_id: &UserId, access_type: UserMetaAccess) -> anyhow::Result<Option<UserInformationModel>>;
    fn set_user_type(connection: &mut PooledConnection, user_id: &UserId, user_type: UserType) -> anyhow::Result<()>;
    fn get_user_type(connection: &mut PooledConnection, user_id: &UserId) -> anyhow::Result<UserType>;
}

impl UserStoreTrait for SqliteStore {
    #[tracing::instrument(name="Update user", skip(connection))]
    fn update_user(connection: &mut PooledConnection, user_id: &UserId, update_request: &UserUpdate) -> anyhow::Result<usize> {
        Ok(diesel::update(user::table.filter(user::id.eq(user_id))).set(update_request).execute(connection)?)
    }

    #[tracing::instrument(name="Get user", skip(connection))]
    fn get_user_information(connection: &mut PooledConnection, user_id: &UserId, access_type: UserMetaAccess) -> anyhow::Result<Option<UserInformationModel>> {
        let result = user::table
            .select((user::id, user::name, user::email, user::device_id, user::custom_id, user::user_type, user::insert_date, user::last_login_date))
            .filter(user::id.eq(&user_id))
            .get_result::<(UserId, Option<String>, Option<String>, Option<String>, Option<String>, i32, i32, i32)>(connection)
            .optional()?;

        match result {
            Some((id, name, email, device_id, custom_id, user_type, insert_date, last_login_date)) => {
                let meta: HashMap<_, _> = Self::get_user_meta(connection, user_id, access_type)?.into_iter().map(|(_, key, value)| (key, value)).collect();
                let meta = match meta.is_empty() {
                    true => None,
                    false => Some(meta)
                };
                Ok(Some(UserInformationModel { id, name, email, device_id, custom_id, meta, user_type: user_type.into(), insert_date, last_login_date, online: false }))
            },
            None => Ok(None)
        }
    }

    #[tracing::instrument(name="Get user meta", skip(connection))]
    fn get_user_meta(connection: &mut PooledConnection, user_id: &UserId, filter: UserMetaAccess) -> anyhow::Result<Vec<(UserMetaId, String, MetaType<UserMetaAccess>)>> {
        let records: Vec<UserMetaModel> = user_meta::table
            .select((user_meta::id, user_meta::key, user_meta::value, user_meta::meta_type, user_meta::access))
            .filter(user_meta::user_id.eq(user_id))
            .filter(user_meta::access.le(i32::from(filter)))
            .load::<UserMetaModel>(connection)?;

        let records = records.into_iter().map(|record| {
            let UserMetaModel { id, key, value, meta_type, access } = record;

            let meta = match meta_type {
                1 => MetaType::Number(value.parse::<f64>().unwrap_or_default(), access.into()),
                2 => MetaType::String(value, access.into()),
                3 => MetaType::Bool(value.parse::<bool>().unwrap_or_default(), access.into()),
                _ => MetaType::String("".to_string(), access.into()),
            };

            (id, key, meta)
        }).collect();
            
        Ok(records)
    }

    #[tracing::instrument(name="Get User type", skip(connection))]
    fn get_user_type(connection: &mut PooledConnection, user_id: &UserId) -> anyhow::Result<UserType> {
        Ok(user::table
            .select(user::user_type)
            .filter(user::id.eq(user_id.borrow()))
            .get_result::<i32>(connection)
            .optional()?
            .map(UserType::from)
            .unwrap_or_default())
    }

    #[tracing::instrument(name="Remove metas", skip(connection))]
    fn remove_user_metas(connection: &mut PooledConnection, ids: Vec<UserMetaId>) -> anyhow::Result<()> {
        diesel::delete(user_meta::table.filter(user_meta::id.eq_any(ids))).execute(connection)?;
        Ok(())
    }

    #[tracing::instrument(name="Insert metas", skip(connection))]
    fn insert_user_metas(connection: &mut PooledConnection, user_id: &UserId, metas: Vec<(String, MetaType<UserMetaAccess>)>) -> anyhow::Result<()> {
        let insert_date = SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default();
        let mut inserts = Vec::new();

        for (key, meta) in metas.into_iter() {
            let id = UserMetaId::default();
            let (value, access, meta_type) = match meta {
                MetaType::Null => continue,
                MetaType::Number(value, access) => (value.to_string(), access, 1),
                MetaType::String(value, access) => (value, access, 2),
                MetaType::Bool(value, access) => (value.to_string(), access, 3),
            };

            let insert = UserMetaInsert {
                id,
                user_id,
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

    #[tracing::instrument(name="Set user type", skip(connection))]
    fn set_user_type(connection: &mut PooledConnection, user_id: &UserId, user_type: UserType) -> anyhow::Result<()> {
        diesel::update(user::table.filter(user::id.eq(user_id.borrow()))).set(user::user_type.eq::<i32>(user_type.into())).execute(connection)?;
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

    #[test]
    fn fail_get_user_information_1() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert!(SqliteStore::get_user_information(&mut connection, &UserId::default(), UserMetaAccess::Anonymous)?.is_none());
        Ok(())
    }

    #[test]
    fn fail_get_user_information_2() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert!(SqliteStore::get_user_information(&mut connection, &UserId::default(), UserMetaAccess::Anonymous)?.is_none());
        Ok(())
    }

    #[test]
    fn get_user_information_1() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let user_id = SqliteStore::create_user_via_email(&mut connection, "erhanbaris@gmail.com", &"erhan".into())?;
        let user = SqliteStore::get_user_information(&mut connection, &user_id, UserMetaAccess::Anonymous)?.unwrap();
        assert_eq!(user.email, Some("erhanbaris@gmail.com".to_string()));
        assert!(user.custom_id.is_none());
        assert!(user.device_id.is_none());
        assert!(user.name.is_none());

        Ok(())
    }

    #[test]
    fn get_user_information_2() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let user_id = SqliteStore::create_user_via_device_id(&mut connection, "123456789")?;
        let user = SqliteStore::get_user_information(&mut connection, &user_id, UserMetaAccess::Anonymous)?.unwrap();
        assert_eq!(user.device_id, Some("123456789".to_string()));
        assert!(user.email.is_none());
        assert!(user.custom_id.is_none());
        assert!(user.name.is_none());

        Ok(())
    }

    #[test]
    fn get_user_information_3() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let user_id = SqliteStore::create_user_via_custom_id(&mut connection, "123456789")?;
        let user = SqliteStore::get_user_information(&mut connection, &user_id, UserMetaAccess::Anonymous)?.unwrap();
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

        assert!(SqliteStore::update_user(&mut connection, &UserId::default(), &UserUpdate::default()).is_err());
        Ok(())
    }

    #[test]
    fn fail_update_user_2() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert!(SqliteStore::update_user(&mut connection, &UserId::default(), &UserUpdate::default()).is_err());
        Ok(())
    }
    #[test]
    fn fail_update_user_3() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert_eq!(SqliteStore::update_user(&mut connection, &UserId::default(), &UserUpdate {
            name: Some(Some("123456".to_string())),
            ..Default::default()
        })?, 0);
        Ok(())
    }

    #[test]
    fn fail_update_user_4() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        assert_eq!(SqliteStore::update_user(&mut connection, &UserId::default(), &UserUpdate {
            name: Some(Some("123456".to_string())),
            ..Default::default()
        })?, 0);
        Ok(())
    }

    #[test]
    fn update_user() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let user_id = SqliteStore::create_user_via_custom_id(&mut connection, "123456789")?;
        assert_eq!(SqliteStore::update_user(&mut connection, &user_id, &UserUpdate {
            name: Some(Some("123456".to_string())),
            ..Default::default()
        })?, 1);
        Ok(())
    }

    #[test]
    fn meta() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let user_id = SqliteStore::create_user_via_custom_id(&mut connection, "123456789")?;
        assert_eq!(SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::System)?.len(), 0);

        // New meta
        SqliteStore::insert_user_metas(&mut connection, &user_id, vec![("gender".to_string(), MetaType::String("male".to_string(), UserMetaAccess::Friend))])?;

        assert_eq!(SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::Friend)?.len(), 1);
        assert_eq!(SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::Anonymous)?.len(), 0);

        let meta = SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::System)?;
        assert_eq!(meta.len(), 1);

        // Remove meta
        SqliteStore::remove_user_metas(&mut connection, vec![meta[0].0.clone()])?;
        assert_eq!(SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::Friend)?.len(), 0);
        assert_eq!(SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::Anonymous)?.len(), 0);
        assert_eq!(SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::System)?.len(), 0);

        SqliteStore::insert_user_metas(&mut connection, &user_id, vec![
            ("location".to_string(), MetaType::String("copenhagen".to_string(), UserMetaAccess::Anonymous)),
            ("score".to_string(), MetaType::Number(123.0, UserMetaAccess::Friend))])?;

        assert_eq!(SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::Friend)?.len(), 2);
        assert_eq!(SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::System)?.len(), 2);

        // Filter with anonymous
        let meta = SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::Anonymous)?;
        assert_eq!(meta.len(), 1);
        assert_eq!(meta.into_iter().map(|(_, key, value)| (key, value)).collect::<Vec<(String, MetaType<UserMetaAccess>)>>(), vec![
            ("location".to_string(), MetaType::String("copenhagen".to_string(), UserMetaAccess::Anonymous))]);

        // Filter with system
        let meta = SqliteStore::get_user_meta(&mut connection, &user_id, UserMetaAccess::System)?;
        assert_eq!(meta.len(), 2);
        assert_eq!(meta.into_iter().map(|(_, key, value)| (key, value)).collect::<Vec<(String, MetaType<UserMetaAccess>)>>(), vec![
            ("location".to_string(), MetaType::String("copenhagen".to_string(), UserMetaAccess::Anonymous)),
            ("score".to_string(), MetaType::Number(123.0, UserMetaAccess::Friend))]);

        Ok(())
    }

}