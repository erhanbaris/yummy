use anyhow::Ok;
use yummy_model::meta::{MetaType, RoomMetaAccess};
use yummy_model::*;
use crate::room::RoomStoreTrait;
use crate::{create_database, create_connection, PooledConnection};

use crate::SqliteStore;

fn db_conection() -> anyhow::Result<PooledConnection> {
    let mut connection = create_connection(":memory:")?.get()?;
    create_database(&mut connection)?;
    Ok(connection)
}

#[test]
fn create_room_1() -> anyhow::Result<()> {
    let mut connection = db_conection()?;
    SqliteStore::create_room(&mut connection, Some("room 1".to_string()), CreateRoomAccessType::Public, 2, false,  &vec!["tag 1".to_string(), "tag 2".to_string()])?;
    Ok(())
}

#[test]
fn create_room_2() -> anyhow::Result<()> {
    let mut connection = db_conection()?;
    let room_1 = SqliteStore::create_room(&mut connection, None, CreateRoomAccessType::Public, 2, false, &Vec::new())?;
    let room_2 = SqliteStore::create_room(&mut connection, None, CreateRoomAccessType::Public, 20, false, &Vec::new())?;

    assert_ne!(room_1, room_2);
    Ok(())
}

#[test]
fn join_to_room() -> anyhow::Result<()> {
    let mut connection = db_conection()?;
    let room = SqliteStore::create_room(&mut connection, None, CreateRoomAccessType::Private, 2, false, &Vec::new())?;
    SqliteStore::join_to_room(&mut connection, &room, &UserId::default(), RoomUserType::User)?;
    Ok(())
}

#[test]
fn join_to_room_request() -> anyhow::Result<()> {
    let mut connection = db_conection()?;
    let room = SqliteStore::create_room(&mut connection, None, CreateRoomAccessType::Private, 2, false, &Vec::new())?;

    let user_1 = UserId::new();
    let user_2 = UserId::new();
    let user_3 = UserId::new();

    SqliteStore::join_to_room_request(&mut connection, &room, &user_1, RoomUserType::User)?;
    SqliteStore::join_to_room_request(&mut connection, &room, &user_2, RoomUserType::Moderator)?;
    SqliteStore::join_to_room_request(&mut connection, &room, &user_3, RoomUserType::Owner)?;
    let mut users = SqliteStore::get_join_requested_users(&mut connection, &room)?;

    assert_eq!(users.len(), 3);
    users.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    assert_eq!(users, vec![(user_1, RoomUserType::User, false), (user_2, RoomUserType::Moderator, false), (user_3, RoomUserType::Owner, false)]);
    Ok(())
}

#[test]
fn disconnect_from_room() -> anyhow::Result<()> {
    let mut connection = db_conection()?;
    let room = SqliteStore::create_room(&mut connection, None, CreateRoomAccessType::Friend, 2, false, &Vec::new())?;
    let user = UserId::default();
    
    assert!(SqliteStore::disconnect_from_room(&mut connection, &room, &user).is_err());
    
    SqliteStore::join_to_room(&mut connection, &room, &user, RoomUserType::User)?;
    SqliteStore::disconnect_from_room(&mut connection, &room, &user)?;

    Ok(())
}

#[test]
fn meta() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    let room = SqliteStore::create_room(&mut connection, None, CreateRoomAccessType::Friend, 2, false, &Vec::new())?;
    
    // New meta
    SqliteStore::insert_room_metas(&mut connection, &room, &vec![
        (&"game-type".to_string(), &MetaType::String("war".to_string(), RoomMetaAccess::Owner)),
        (&"players".to_string(), &MetaType::List(Box::new(vec![MetaType::Number(12345.0, RoomMetaAccess::Owner), MetaType::Number(67890.0, RoomMetaAccess::Owner)]), RoomMetaAccess::Owner))])?;


    let meta = SqliteStore::get_room_meta(&mut connection, &room, RoomMetaAccess::System)?;
    assert_eq!(meta.len(), 2);

    // Remove meta
    SqliteStore::remove_room_metas(&mut connection, vec![meta[0].0.clone()])?;
    let meta = SqliteStore::get_room_meta(&mut connection, &room, RoomMetaAccess::Owner)?;
    assert_eq!(meta[0].1, "players".to_string());
    assert_eq!(meta[0].2, MetaType::List(Box::new(vec![MetaType::Number(12345.0, RoomMetaAccess::Anonymous), MetaType::Number(67890.0, RoomMetaAccess::Anonymous)]), RoomMetaAccess::Owner));

    assert_eq!(SqliteStore::get_room_meta(&mut connection, &room, RoomMetaAccess::Owner)?.len(), 1);
    assert_eq!(SqliteStore::get_room_meta(&mut connection, &room, RoomMetaAccess::Anonymous)?.len(), 0);
    assert_eq!(SqliteStore::get_room_meta(&mut connection, &room, RoomMetaAccess::System)?.len(), 1);

    SqliteStore::insert_room_metas(&mut connection, &room, &vec![
        (&"location".to_string(), &MetaType::String("copenhagen".to_string(), RoomMetaAccess::Anonymous)),
        (&"score".to_string(), &MetaType::Number(123.0, RoomMetaAccess::Owner))])?;

    assert_eq!(SqliteStore::get_room_meta(&mut connection, &room, RoomMetaAccess::Owner)?.len(), 3);
    assert_eq!(SqliteStore::get_room_meta(&mut connection, &room, RoomMetaAccess::System)?.len(), 3);

    // Filter with anonymous
    let meta = SqliteStore::get_room_meta(&mut connection, &room, RoomMetaAccess::Anonymous)?;
    assert_eq!(meta.len(), 1);
    assert_eq!(meta.into_iter().map(|(_, key, value)| (key, value)).collect::<Vec<(String, MetaType<RoomMetaAccess>)>>(), vec![
        ("location".to_string(), MetaType::String("copenhagen".to_string(), RoomMetaAccess::Anonymous))]);

    // Filter with system
    let meta = SqliteStore::get_room_meta(&mut connection, &room, RoomMetaAccess::System)?;
    assert_eq!(meta.len(), 3);
    assert_eq!(meta.into_iter().map(|(_, key, value)| (key, value)).collect::<Vec<(String, MetaType<RoomMetaAccess>)>>(), vec![
        ("players".to_string(), MetaType::List(Box::new(vec![MetaType::Number(12345.0, RoomMetaAccess::Anonymous), MetaType::Number(67890.0, RoomMetaAccess::Anonymous)]), RoomMetaAccess::Owner)),
        ("location".to_string(), MetaType::String("copenhagen".to_string(), RoomMetaAccess::Anonymous)),
        ("score".to_string(), MetaType::Number(123.0, RoomMetaAccess::Owner))]);

    Ok(())
}

#[test]
fn ban_user() -> anyhow::Result<()> {
    let mut connection = db_conection()?;

    let room = SqliteStore::create_room(&mut connection, None, CreateRoomAccessType::Friend, 2, false, &Vec::new())?;
    SqliteStore::ban_user_from_room(&mut connection, &room, &UserId::default(), &UserId::default())?;

    Ok(())
}
