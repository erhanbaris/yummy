use diesel::{SqliteConnection, r2d2::ConnectionManager};

pub type ConnectionType = SqliteConnection;
pub type Pool = r2d2::Pool<ConnectionManager<ConnectionType>>;
pub type PooledConnection = ::r2d2::PooledConnection<ConnectionManager<ConnectionType>>;