use std::{env::temp_dir, sync::Arc};

use yummy_database::{create_connection, create_database};

use yummy_general::database::Pool;
use uuid::Uuid;

pub fn get_database_pool() -> Arc<Pool>{
    let mut db_location = temp_dir();
    db_location.push(format!("{}.db", Uuid::new_v4()));

    let connection = create_connection(db_location.to_str().unwrap()).unwrap();
    create_database(&mut connection.get().unwrap()).unwrap();
    Arc::new(connection)
}