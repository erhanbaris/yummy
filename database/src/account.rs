use crate::{PooledConnection, RowId};

pub enum UpdateAccountFieldType {
    Name,
    Password,
    DeviceId,
    CustomId,
    Email
}

pub struct UpdateAccountField<'a> {
    field: UpdateAccountFieldType,
    value: &'a str
}

pub struct AccountStore {
    database: PooledConnection
}

pub trait AccountStoreTrait {
    fn new(database: PooledConnection) -> Self;

    fn update_account<'a>(&mut self, user_id: RowId, fields: &[UpdateAccountField]);
}

impl AccountStoreTrait for AccountStore {
    fn new(database: PooledConnection) -> Self {
        Self { database }
    }

    fn update_account<'a>(&mut self, _: RowId, _: &[UpdateAccountField]) {
        
    }
}