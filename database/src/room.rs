use std::time::{SystemTime, UNIX_EPOCH};

use diesel::RunQueryDsl;
use general::model::{CreateRoomAccessType, RoomUserType};

use crate::{SqliteStore, PooledConnection, RowId, model::{RoomInsert, RoomTagInsert, RoomUserInsert}, schema::{room::{self}, room_tag, room_user}};

pub trait RoomStoreTrait: Sized {
    fn create_room(connection: &mut PooledConnection, name: Option<String>, access_type: CreateRoomAccessType, password: Option<&str>, max_user: usize, tags: Vec<String>) -> anyhow::Result<RowId>;
    fn join_to_room(connection: &mut PooledConnection, room_id: RowId, user_id: RowId, user_type: RoomUserType) -> anyhow::Result<()>;
}

impl RoomStoreTrait for SqliteStore {
    #[tracing::instrument(name="Create room", skip(connection))]
    fn create_room<'a>(connection: &mut PooledConnection, name: Option<String>, access_type: CreateRoomAccessType, password: Option<&'a str>, max_user: usize, tags: Vec<String>) -> anyhow::Result<RowId> {
        let insert_date = SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default();

        let mut model = RoomInsert {
            insert_date,
            password,
            name,
            max_user: max_user as i32,
            ..Default::default()
        };

        
        let (access, supplementary) = match access_type {
            CreateRoomAccessType::Public => (1, None),
            CreateRoomAccessType::Private => (2, None),
            CreateRoomAccessType::Friend => (3, None),
            CreateRoomAccessType::Tag(tag) => (4, Some(tag)),
        };
        model.access_type = access;
        model.access_supplementary = supplementary;
        
        let room_id = model.id;
        diesel::insert_into(room::table).values(&vec![model]).execute(connection)?;

        let mut tag_inserts = Vec::new();
        for tag in tags.iter() {
            let insert = RoomTagInsert {
                room_id,
                tag: &tag[..],
                insert_date,
                ..Default::default()
            };
            tag_inserts.push(insert);
        }

        if !tag_inserts.is_empty() {
            diesel::insert_into(room_tag::table).values(&tag_inserts).execute(connection)?;
        }

        Ok(room_id)
    }

    #[tracing::instrument(name="Join to room", skip(connection))]
    fn join_to_room(connection: &mut PooledConnection, room_id: RowId, user_id: RowId, user_type: RoomUserType) -> anyhow::Result<()> {

        let insert = RoomUserInsert {
            insert_date: SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default(),
            room_id,
            user_id,
            room_user_type: user_type as i32,
            ..Default::default()
        };
        diesel::insert_into(room_user::table).values(&vec![insert]).execute(connection)?;

        Ok(())
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

    #[test]
    fn create_room_1() -> anyhow::Result<()> {
        let mut connection = db_conection()?;
        SqliteStore::create_room(&mut connection, Some("room 1".to_string()), CreateRoomAccessType::Public, Some("password"), 2, vec!["tag 1".to_string(), "tag 2".to_string()])?;
        Ok(())
    }

    #[test]
    fn create_room_2() -> anyhow::Result<()> {
        let mut connection = db_conection()?;
        let room_1 = SqliteStore::create_room(&mut connection, None, CreateRoomAccessType::Tag("LEVEL 1000".to_string()), None, 2, Vec::new())?;
        let room_2 = SqliteStore::create_room(&mut connection, None, CreateRoomAccessType::Public, None, 20, Vec::new())?;

        assert_ne!(room_1, room_2);
        Ok(())
    }

    #[test]
    fn join_to_room() -> anyhow::Result<()> {
        let mut connection = db_conection()?;
        let room = SqliteStore::create_room(&mut connection, None, CreateRoomAccessType::Tag("LEVEL 1000".to_string()), None, 2, Vec::new())?;
        SqliteStore::join_to_room(&mut connection, room, RowId::default(), RoomUserType::User)?;
        Ok(())
    }
}