use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use diesel::RunQueryDsl;
use diesel::QueryDsl;
use diesel::ExpressionMethods;
use general::meta::RoomMetaAccess;
use general::meta::MetaType;
use general::model::RoomId;
use general::model::RoomMetaId;
use general::model::RoomTagId;
use general::model::RoomUserId;
use general::model::UserId;
use general::model::UserMetaId;
use general::model::{CreateRoomAccessType, RoomUserType};

use crate::model::{RoomMetaInsert, RoomUserRequestInsert};
use crate::model::RoomMetaModel;
use crate::model::RoomUpdate;
use crate::schema::{room_meta, room_user_request};
use crate::{SqliteStore, PooledConnection, model::{RoomInsert, RoomTagInsert, RoomUserInsert}, schema::{room::{self}, room_tag, room_user}};

pub trait RoomStoreTrait: Sized {
    fn create_room(connection: &mut PooledConnection, name: Option<String>, access_type: CreateRoomAccessType, max_user: usize, join_request: bool, tags: &[String]) -> anyhow::Result<RoomId>;
    fn join_to_room(connection: &mut PooledConnection, room_id: &RoomId, user_id: &UserId, user_type: RoomUserType) -> anyhow::Result<()>;
    fn join_to_room_request(connection: &mut PooledConnection, room_id: &RoomId, user_id: &UserId, user_type: RoomUserType) -> anyhow::Result<()>;
    fn update_join_to_room_request(connection: &mut PooledConnection, room_id: &RoomId, user_id: &UserId, updater_user_id: &UserId, status: bool) -> anyhow::Result<()>;
    fn disconnect_from_room(connection: &mut PooledConnection, room_id: &RoomId, user_id: &UserId) -> anyhow::Result<()>;
    fn get_join_requested_users(connection: &mut PooledConnection, room_id: &RoomId) -> anyhow::Result<Vec<(UserId, RoomUserType, bool)>>;
    fn get_room_meta(connection: &mut PooledConnection, room_id: &RoomId, filter: RoomMetaAccess) -> anyhow::Result<Vec<(RoomMetaId, String, MetaType<RoomMetaAccess>)>>;
    fn get_room_tag(connection: &mut PooledConnection, room_id: &RoomId) -> anyhow::Result<Vec<(RoomTagId, String)>>;
    fn remove_room_tags(connection: &mut PooledConnection, ids: Vec<RoomTagId>) -> anyhow::Result<()>;
    fn remove_room_metas(connection: &mut PooledConnection, ids: Vec<RoomMetaId>) -> anyhow::Result<()>;
    fn insert_room_metas(connection: &mut PooledConnection, room_id: &RoomId, metas: &Vec<(&String, &MetaType<RoomMetaAccess>)>) -> anyhow::Result<()>;
    fn insert_room_tags(connection: &mut PooledConnection, room_id: &RoomId, tags: &Vec<String>) -> anyhow::Result<()>;
    fn update_room(connection: &mut PooledConnection, room_id: &RoomId, update_request: &RoomUpdate) -> anyhow::Result<usize>;
    fn update_room_user_permissions(connection: &mut PooledConnection, room_id: &RoomId, permissions: &HashMap<UserId, RoomUserType>) -> anyhow::Result<()>;
}

impl RoomStoreTrait for SqliteStore {
    #[tracing::instrument(name="Create room", skip(connection))]
    fn create_room(connection: &mut PooledConnection, name: Option<String>, access_type: CreateRoomAccessType, max_user: usize, join_request: bool, tags: &[String]) -> anyhow::Result<RoomId> {
        let insert_date = SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default();

        let mut model = RoomInsert {
            id: RoomId::default(),
            access_type: 0,
            insert_date,
            name,
            join_request: join_request as i32,
            max_user: max_user as i32
        };
        
        let access = match access_type {
            CreateRoomAccessType::Public => 1,
            CreateRoomAccessType::Private => 2,
            CreateRoomAccessType::Friend => 3
        };
        model.access_type = access;
        
        let room_id = model.id;
        let affected_rows = diesel::insert_into(room::table).values(&vec![model]).execute(connection)?;
        if affected_rows == 0 {
            return Err(anyhow::anyhow!("No row inserted"));
        }

        let mut tag_inserts = Vec::new();
        for tag in tags.iter() {
            let insert = RoomTagInsert {
                room_id: &room_id,
                tag: &tag[..],
                insert_date,
                id: RoomTagId::default()
            };
            tag_inserts.push(insert);
        }

        if !tag_inserts.is_empty() {
            diesel::insert_into(room_tag::table).values(&tag_inserts).execute(connection)?;
        }

        Ok(room_id)
    }

    fn insert_room_tags(connection: &mut PooledConnection, room_id: &RoomId, tags: &Vec<String>) -> anyhow::Result<()> {
        let insert_date = SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default();

        let mut tag_inserts = Vec::new();
        for tag in tags.iter() {
            let insert = RoomTagInsert {
                room_id: &room_id,
                tag: &tag[..],
                insert_date,
                id: RoomTagId::default()
            };
            tag_inserts.push(insert);
        }

        if !tag_inserts.is_empty() {
            diesel::insert_into(room_tag::table).values(&tag_inserts).execute(connection)?;
        }

        Ok(())
    }
    
    #[tracing::instrument(name="Update user", skip(connection))]
    fn update_room(connection: &mut PooledConnection, room_id: &RoomId, update_request: &RoomUpdate) -> anyhow::Result<usize> {
        Ok(diesel::update(room::table.filter(room::id.eq(room_id))).set(update_request).execute(connection)?)
    }

    #[tracing::instrument(name="update room user permissions", skip(connection))]
    fn update_room_user_permissions(connection: &mut PooledConnection, room_id: &RoomId, permissions: &HashMap<UserId, RoomUserType>) -> anyhow::Result<()> {
        for (user_id, permission) in permissions.into_iter() {
            diesel::update(room_user::table.filter(room_user::room_id.eq(room_id))).set((room_user::user_id.eq(user_id), room_user::room_user_type.eq(permission.clone() as i32))).execute(connection)?;
        }
        Ok(())
    }

    #[tracing::instrument(name="Join to room", skip(connection))]
    fn join_to_room(connection: &mut PooledConnection, room_id: &RoomId, user_id: &UserId, user_type: RoomUserType) -> anyhow::Result<()> {

        let insert = RoomUserInsert {
            id: RoomUserId::default(),
            insert_date: SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default(),
            room_id,
            user_id,
            room_user_type: user_type as i32
        };
        
        let affected_rows = diesel::insert_into(room_user::table).values(&vec![insert]).execute(connection)?;
        if affected_rows == 0 {
            return Err(anyhow::anyhow!("No row inserted"));
        }
        Ok(())
    }

    #[tracing::instrument(name="Join to room request", skip(connection))]
    fn join_to_room_request(connection: &mut PooledConnection, room_id: &RoomId, user_id: &UserId, user_type: RoomUserType) -> anyhow::Result<()> {

        let insert = RoomUserRequestInsert {
            id: RoomUserId::default(),
            insert_date: SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default(),
            room_id,
            user_id,
            room_user_type: user_type as i32,
            status_updater_user_id: None,
            status: false
        };
        
        let affected_rows = diesel::insert_into(room_user_request::table).values(&vec![insert]).execute(connection)?;
        if affected_rows == 0 {
            return Err(anyhow::anyhow!("No row inserted"));
        }
        Ok(())
    }

    #[tracing::instrument(name="Update join to room request", skip(connection))]
    fn update_join_to_room_request(connection: &mut PooledConnection, room_id: &RoomId, user_id: &UserId, updater_user_id: &UserId, status: bool) -> anyhow::Result<()> {
        diesel::update(room_user_request::table.filter(room_user_request::room_id.eq(room_id))).set((room_user_request::status.eq(status), room_user_request::status_updater_user_id.eq(updater_user_id))).execute(connection)?;
        Ok(())
    }

    #[tracing::instrument(name="Join requested users", skip(connection))]
    fn get_join_requested_users(connection: &mut PooledConnection, room_id: &RoomId) -> anyhow::Result<Vec<(UserId, RoomUserType, bool)>> {
        let records: Vec<(UserId, i32, bool)> = room_user_request::table
            .select((room_user_request::user_id, room_user_request::room_user_type, room_user_request::status))
            .filter(room_user_request::room_id.eq(room_id))
            .load(connection)?;
        
        let records = records.into_iter().map(|(user_id, room_user_type, status)| {
            let room_user_type = match room_user_type {
                1 => RoomUserType::User,
                2 => RoomUserType::Moderator,
                3 => RoomUserType::Owner,
                _ => RoomUserType::User,
            };

            (user_id, room_user_type, status)
        }).collect();
        
        Ok(records)
    }

    #[tracing::instrument(name="Disconnect from room", skip(connection))]
    fn disconnect_from_room(connection: &mut PooledConnection, room_id: &RoomId, user_id: &UserId) -> anyhow::Result<()> {
        let affected_rows = diesel::delete(room_user::table.filter(room_user::user_id.eq(&user_id)).filter(room_user::room_id.eq(&room_id))).execute(connection)?;
        if affected_rows == 0 {
            return Err(anyhow::anyhow!("No row removed"));
        }
        Ok(())
    }

    #[tracing::instrument(name="Get room meta", skip(connection))]
    fn get_room_meta(connection: &mut PooledConnection, room_id: &RoomId, filter: RoomMetaAccess) -> anyhow::Result<Vec<(RoomMetaId, String, MetaType<RoomMetaAccess>)>> {
        let records: Vec<RoomMetaModel> = room_meta::table
            .select((room_meta::id, room_meta::key, room_meta::value, room_meta::meta_type, room_meta::access))
            .filter(room_meta::room_id.eq(room_id))
            .filter(room_meta::access.le(i32::from(filter)))
            .load::<RoomMetaModel>(connection)?;

        let records = records.into_iter().map(|record| {
            let RoomMetaModel { id, key, value, meta_type, access } = record;

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

    #[tracing::instrument(name="Get room tags", skip(connection))]
    fn get_room_tag(connection: &mut PooledConnection, room_id: &RoomId) -> anyhow::Result<Vec<(RoomTagId, String)>> {
        let records: Vec<(RoomTagId, String)> = room_tag::table
            .select((room_tag::id, room_tag::tag))
            .filter(room_tag::room_id.eq(room_id))
            .get_results::<(RoomTagId, String)>(connection)?;

        Ok(records)
    }

    #[tracing::instrument(name="Remove room tags", skip(connection))]
    fn remove_room_tags(connection: &mut PooledConnection, ids: Vec<RoomTagId>) -> anyhow::Result<()> {
        diesel::delete(room_tag::table.filter(room_tag::id.eq_any(ids))).execute(connection)?;
        Ok(())
    }

    #[tracing::instrument(name="Remove metas", skip(connection))]
    fn remove_room_metas(connection: &mut PooledConnection, ids: Vec<RoomMetaId>) -> anyhow::Result<()> {
        diesel::delete(room_meta::table.filter(room_meta::id.eq_any(ids))).execute(connection)?;
        Ok(())
    }

    #[tracing::instrument(name="Insert metas", skip(connection))]
    fn insert_room_metas(connection: &mut PooledConnection, room_id: &RoomId, metas: &Vec<(&String, &MetaType<RoomMetaAccess>)>) -> anyhow::Result<()> {
        let insert_date = SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default();
        let mut inserts = Vec::new();

        for (key, meta) in metas.iter() {
            let (value, access, meta_type) = match meta {
                MetaType::Null => continue,
                MetaType::Number(value, access) => (value.to_string(), access, 1),
                MetaType::String(value, access) => (value.clone(), access, 2),
                MetaType::Bool(value, access) => (value.to_string(), access, 3),
            };

            let insert = RoomMetaInsert {
                id: UserMetaId::default(),
                room_id,
                key,
                value,
                access: i32::from(access.clone()),
                meta_type,
                insert_date
            };

            inserts.push(insert);
        }
        diesel::insert_into(room_meta::table).values(&inserts).execute(connection)?;
        Ok(())
    }
}
