use std::borrow::Borrow;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Deref;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use diesel::QueryDsl;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use diesel::result::OptionalExtension;
use general::meta::MetaType;
use general::meta::UserMetaAccess;
use general::model::UserId;
use general::model::UserInformationModel;
use general::model::UserMetaId;
use general::model::UserType;

use crate::SqliteStore;
use crate::model::UserMetaInsert;
use crate::model::UserMetaModel;
use crate::model::UserUpdate;
use crate::schema::user_meta;
use crate::{PooledConnection, schema::user};

pub trait UserStoreTrait: Sized {
    fn update_user(connection: &mut PooledConnection, user_id: &UserId, update_request: &UserUpdate) -> anyhow::Result<usize>;
    fn get_user_meta(connection: &mut PooledConnection, user_id: &UserId, filter: UserMetaAccess) -> anyhow::Result<Vec<(UserMetaId, String, MetaType<UserMetaAccess>)>>;
    fn remove_user_metas(connection: &mut PooledConnection, meta_ids: Vec<UserMetaId>) -> anyhow::Result<()>;
    fn insert_user_metas<'a>(connection: &mut PooledConnection, user_id: &UserId, metas: Vec<(&'a String, &'a MetaType<UserMetaAccess>)>) -> anyhow::Result<()>;
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
                let metas: HashMap<_, _> = Self::get_user_meta(connection, user_id, access_type)?.into_iter().map(|(_, key, value)| (key, value)).collect();
                let metas = match metas.is_empty() {
                    true => None,
                    false => Some(metas)
                };
                Ok(Some(UserInformationModel { id, name, email, device_id, custom_id, metas, user_type: user_type.into(), insert_date, last_login_date, online: false }))
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
                4 => MetaType::List(Box::new(serde_json::from_str(&value[..]).unwrap_or_default()), access.into()),
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
    fn insert_user_metas<'a>(connection: &mut PooledConnection, user_id: &UserId, metas: Vec<(&'a String, &'a MetaType<UserMetaAccess>)>) -> anyhow::Result<()> {
        let insert_date = SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default();
        let mut inserts = Vec::new();

        for (key, meta) in metas.into_iter() {
            let id = UserMetaId::default();
            let (value, access, meta_type) = match meta {
                MetaType::Null => continue,
                MetaType::Number(value, access) => (Cow::Owned(value.to_string()), access, 1),
                MetaType::String(value, access) => (Cow::Borrowed(value), access, 2),
                MetaType::Bool(value, access) => (Cow::Owned(value.to_string()), access, 3),
                MetaType::List(value, access) => (Cow::Owned(serde_json::to_string(value.deref()).unwrap_or_default()), access, 4),
            };

            let insert = UserMetaInsert {
                id,
                user_id,
                key,
                value,
                access: access.clone().into(),
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
