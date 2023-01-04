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

use crate::model::RoomMetaInsert;
use crate::model::RoomMetaModel;
use crate::model::RoomUpdate;
use crate::schema::room_meta;
use crate::{SqliteStore, PooledConnection, model::{RoomInsert, RoomTagInsert, RoomUserInsert}, schema::{room::{self}, room_tag, room_user}};

pub trait RoomStoreTrait: Sized {
    fn create_room(connection: &mut PooledConnection, name: Option<String>, access_type: CreateRoomAccessType, max_user: usize, tags: &[String]) -> anyhow::Result<RoomId>;
    fn join_to_room(connection: &mut PooledConnection, room_id: &RoomId, user_id: &UserId, user_type: RoomUserType) -> anyhow::Result<()>;
    fn disconnect_from_room(connection: &mut PooledConnection, room_id: &RoomId, user_id: &UserId) -> anyhow::Result<()>;
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
    fn create_room(connection: &mut PooledConnection, name: Option<String>, access_type: CreateRoomAccessType, max_user: usize, tags: &[String]) -> anyhow::Result<RoomId> {
        let insert_date = SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default();

        let mut model = RoomInsert {
            id: RoomId::default(),
            access_type: 0,
            insert_date,
            name,
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

#[cfg(test)]
mod tests {
    use anyhow::Ok;
    use general::meta::{MetaType, RoomMetaAccess};
    use crate::{create_database, create_connection, PooledConnection};

    use crate::SqliteStore;
    use super::*;
    fn db_conection() -> anyhow::Result<PooledConnection> {
        let mut connection = create_connection(":memory:")?.get()?;
        create_database(&mut connection)?;
        Ok(connection)
    }

    #[test]
    fn create_room_1() -> anyhow::Result<()> {
        let mut connection = db_conection()?;
        SqliteStore::create_room(&mut connection, Some("room 1".to_string()), CreateRoomAccessType::Public, 2, &vec!["tag 1".to_string(), "tag 2".to_string()])?;
        Ok(())
    }

    #[test]
    fn create_room_2() -> anyhow::Result<()> {
        let mut connection = db_conection()?;
        let room_1 = SqliteStore::create_room(&mut connection, None, CreateRoomAccessType::Public, 2, &Vec::new())?;
        let room_2 = SqliteStore::create_room(&mut connection, None, CreateRoomAccessType::Public, 20, &Vec::new())?;

        assert_ne!(room_1, room_2);
        Ok(())
    }

    #[test]
    fn join_to_room() -> anyhow::Result<()> {
        let mut connection = db_conection()?;
        let room = SqliteStore::create_room(&mut connection, None, CreateRoomAccessType::Private, 2, &Vec::new())?;
        SqliteStore::join_to_room(&mut connection, &room, &UserId::default(), RoomUserType::User)?;
        Ok(())
    }

    #[test]
    fn disconnect_from_room() -> anyhow::Result<()> {
        let mut connection = db_conection()?;
        let room = SqliteStore::create_room(&mut connection, None, CreateRoomAccessType::Friend, 2, &Vec::new())?;
        let user = UserId::default();
        
        assert!(SqliteStore::disconnect_from_room(&mut connection, &room, &user).is_err());
        
        SqliteStore::join_to_room(&mut connection, &room, &user, RoomUserType::User)?;
        SqliteStore::disconnect_from_room(&mut connection, &room, &user)?;

        Ok(())
    }

    #[test]
    fn meta() -> anyhow::Result<()> {
        let mut connection = db_conection()?;

        let room = SqliteStore::create_room(&mut connection, None, CreateRoomAccessType::Friend, 2, &Vec::new())?;
        
        // New meta
        SqliteStore::insert_room_metas(&mut connection, &room, &vec![(&"game-type".to_string(), &MetaType::String("war".to_string(), RoomMetaAccess::Owner))])?;


        let meta = SqliteStore::get_room_meta(&mut connection, &room, RoomMetaAccess::System)?;
        assert_eq!(meta.len(), 1);

        // Remove meta
        SqliteStore::remove_room_metas(&mut connection, vec![meta[0].0.clone()])?;
        assert_eq!(SqliteStore::get_room_meta(&mut connection, &room, RoomMetaAccess::Owner)?.len(), 0);
        assert_eq!(SqliteStore::get_room_meta(&mut connection, &room, RoomMetaAccess::Anonymous)?.len(), 0);
        assert_eq!(SqliteStore::get_room_meta(&mut connection, &room, RoomMetaAccess::System)?.len(), 0);

        SqliteStore::insert_room_metas(&mut connection, &room, &vec![
            (&"location".to_string(), &MetaType::String("copenhagen".to_string(), RoomMetaAccess::Anonymous)),
            (&"score".to_string(), &MetaType::Number(123.0, RoomMetaAccess::Owner))])?;

        assert_eq!(SqliteStore::get_room_meta(&mut connection, &room, RoomMetaAccess::Owner)?.len(), 2);
        assert_eq!(SqliteStore::get_room_meta(&mut connection, &room, RoomMetaAccess::System)?.len(), 2);

        // Filter with anonymous
        let meta = SqliteStore::get_room_meta(&mut connection, &room, RoomMetaAccess::Anonymous)?;
        assert_eq!(meta.len(), 1);
        assert_eq!(meta.into_iter().map(|(_, key, value)| (key, value)).collect::<Vec<(String, MetaType<RoomMetaAccess>)>>(), vec![
            ("location".to_string(), MetaType::String("copenhagen".to_string(), RoomMetaAccess::Anonymous))]);

        // Filter with system
        let meta = SqliteStore::get_room_meta(&mut connection, &room, RoomMetaAccess::System)?;
        assert_eq!(meta.len(), 2);
        assert_eq!(meta.into_iter().map(|(_, key, value)| (key, value)).collect::<Vec<(String, MetaType<RoomMetaAccess>)>>(), vec![
            ("location".to_string(), MetaType::String("copenhagen".to_string(), RoomMetaAccess::Anonymous)),
            ("score".to_string(), MetaType::Number(123.0, RoomMetaAccess::Owner))]);

        Ok(())
    }
}