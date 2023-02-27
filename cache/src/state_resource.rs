use std::{sync::Arc, marker::PhantomData};

use database::DatabaseTrait;
use general::{config::YummyConfig, database::Pool, meta::UserMetaAccess, model::{UserInformationModel, UserId}};

use crate::{cache::YummyCacheResource, state::resource::YummyCacheResourceFactory};

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

pub struct ResourceFactory<DB: DatabaseTrait + ?Sized> {
    config: Arc<YummyConfig>,
    database: Arc<Pool>,
    _marker: PhantomData<DB>
}

impl<DB: DatabaseTrait + ?Sized> ResourceFactory<DB> {
    pub fn new(config: Arc<YummyConfig>, database: Arc<Pool>) -> Self {
        Self {
            config,
            database,
            _marker: PhantomData
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + 'static> YummyCacheResourceFactory for ResourceFactory<DB> {
    fn user_information(&self) -> Box<dyn YummyCacheResource<K=UserId, V=UserInformationModel>> {
        Box::new(UserInformationResource::<DB>::new(self.config.clone(), self.database.clone()))
    }
}

impl<DB: DatabaseTrait + ?Sized> YummyCacheResource for UserInformationResource<DB> {
    type K=UserId;
    type V=UserInformationModel;

    fn get(&self, key: &Self::K) -> anyhow::Result<Option<Self::V>> {
        let mut connection = self.database.get()?;
        DB::get_user_information(&mut connection, key, UserMetaAccess::System)
    }
}
