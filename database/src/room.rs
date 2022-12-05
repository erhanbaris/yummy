use std::time::{SystemTime, UNIX_EPOCH};

use diesel::RunQueryDsl;
use general::model::{CreateRoomAccessType, RoomUserType};

use crate::{SqliteStore, PooledConnection, RowId, model::{RoomInsert, RoomTagInsert}, schema::{room::{self}, room_tag}};

pub trait RoomStoreTrait: Sized {
    fn create_room<'a>(connection: &mut PooledConnection, name: Option<String>, access_type: CreateRoomAccessType, password: Option<&'a str>, max_user: usize, tags: Vec<String>) -> anyhow::Result<RowId>;
    fn join_to_room(connection: &mut PooledConnection, room_id: RowId, user_id: RowId, user_type: RoomUserType) -> anyhow::Result<()>;
}

impl RoomStoreTrait for SqliteStore {
    #[tracing::instrument(name="Create room", skip(connection))]
    fn create_room<'a>(connection: &mut PooledConnection, name: Option<String>, access_type: CreateRoomAccessType, password: Option<&'a str>, max_user: usize, tags: Vec<String>) -> anyhow::Result<RowId> {
        let insert_date = SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as i32).unwrap_or_default();

        let mut model = RoomInsert::default();
        model.insert_date = insert_date;
        model.password = password;
        model.name = name;
        model.max_user = max_user as i32;
        
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
            let mut insert = RoomTagInsert::default();
            insert.room_id = room_id.clone();
            insert.tag = &tag[..];
            insert.insert_date = insert_date;
            tag_inserts.push(insert);
        }

        if tag_inserts.len() > 0 {
            diesel::insert_into(room_tag::table).values(&tag_inserts).execute(connection)?;
        }

        Ok(room_id)
    }

    #[tracing::instrument(name="Join to room", skip(connection))]
    fn join_to_room(connection: &mut PooledConnection, room_id: RowId, user_id: RowId, user_type: RoomUserType) -> anyhow::Result<()> {
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
        SqliteStore::create_room(&mut connection, None, CreateRoomAccessType::Tag("LEVEL 1000".to_string()), None, 2, Vec::new())?;
        Ok(())
    }
}