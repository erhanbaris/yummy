use std::{sync::Arc, marker::PhantomData};

use general::{config::YummyConfig, database::Pool, meta::UserMetaAccess, model::{UserInformationModel, UserId}, state::resource::StateResourceTrait, cache::YummyCacheResource};

use crate::DatabaseTrait;

pub struct UserInformationResource<DB: DatabaseTrait + ?Sized> {
    config: Arc<YummyConfig>,
    database: Arc<Pool>,
    _marker: PhantomData<DB>
}

impl<DB: DatabaseTrait + ?Sized> UserInformationResource<DB> {
    pub fn new(config: Arc<YummyConfig>, database: Arc<Pool>) -> Self {
        Self {
            config,
            database,
            _marker: PhantomData
        }
    }
}

impl<DB: DatabaseTrait + ?Sized> YummyCacheResource for UserInformationResource<DB> {
    type K=UserId;
    type V=UserInformationModel;

    fn get(&self, key: &Self::K) -> anyhow::Result<Option<Self::V>> {
        let mut connection = self.database.get()?;
        DB::get_user_information(&mut connection, key, UserMetaAccess::System)
    }

    fn set(&self, key: Self::K, value: Self::V) -> anyhow::Result<()> { Ok(()) }
}
