use std::{sync::Arc, marker::PhantomData};

use database::DatabaseTrait;
use general::database::Pool;
use model::{config::YummyConfig, UserId, UserInformationModel, meta::{UserMetaAccess, MetaType}, UserMetaId};

use crate::{cache::YummyCacheResource, state::resource::YummyCacheResourceFactory};

/* User meta resources */
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

    fn user_metas(&self) -> Box<dyn YummyCacheResource<K=UserId, V=Vec<UserMetaInformation>>> {
        Box::new(UserMetaResource::<DB>::new(self.config.clone(), self.database.clone()))
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

/* User meta resources */
#[derive(Clone)]
pub struct UserMetaInformation {
    pub id: UserMetaId,
    pub name: String,
    pub meta: MetaType<UserMetaAccess>
}

pub struct UserMetaResource<DB: DatabaseTrait + ?Sized> {
    config: Arc<YummyConfig>,
    database: Arc<Pool>,
    _marker: PhantomData<DB>
}

impl<DB: DatabaseTrait + ?Sized> UserMetaResource<DB> {
    pub fn new(config: Arc<YummyConfig>, database: Arc<Pool>) -> Self {
        Self {
            config,
            database,
            _marker: PhantomData
        }
    }
}

impl<DB: DatabaseTrait + ?Sized> YummyCacheResource for UserMetaResource<DB> {
    type K=UserId;
    type V=Vec<UserMetaInformation>;

    fn get(&self, key: &Self::K) -> anyhow::Result<Option<Self::V>> {
        let mut connection = self.database.get()?;
        let result = DB::get_user_meta(&mut connection, key, UserMetaAccess::System)?.into_iter().map(|(id, name, meta)| {
            UserMetaInformation {
                id,
                meta,
                name
            }
        }).collect();
        Ok(Some(result))
    }
}
