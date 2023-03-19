/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::{sync::Arc, marker::PhantomData};

use yummy_database::DatabaseTrait;
use yummy_general::database::Pool;
use yummy_model::{UserId, UserInformationModel, meta::{UserMetaAccess, collection::UserMetaCollection}, UserType};

use crate::{cache::YummyCacheResource, state::resource::YummyCacheResourceFactory};

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
pub struct UserInformationResource<DB: DatabaseTrait + ?Sized> {
    database: Arc<Pool>,
    _marker: PhantomData<DB>
}

pub struct ResourceFactory<DB: DatabaseTrait + ?Sized> {
    database: Arc<Pool>,
    _marker: PhantomData<DB>
}

pub struct UserMetaResource<DB: DatabaseTrait + ?Sized> {
    database: Arc<Pool>,
    _marker: PhantomData<DB>
}

pub struct UserTypeResource<DB: DatabaseTrait + ?Sized> {
    database: Arc<Pool>,
    _marker: PhantomData<DB>
}

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* **************************************************************************************************************** */

impl<DB: DatabaseTrait + ?Sized> UserInformationResource<DB> {
    pub fn new(database: Arc<Pool>) -> Self {
        Self {
            database,
            _marker: PhantomData
        }
    }
}

impl<DB: DatabaseTrait + ?Sized> ResourceFactory<DB> {
    pub fn new(database: Arc<Pool>) -> Self {
        Self {
            database,
            _marker: PhantomData
        }
    }
}

impl<DB: DatabaseTrait + ?Sized> UserMetaResource<DB> {
    pub fn new(database: Arc<Pool>) -> Self {
        Self {
            database,
            _marker: PhantomData
        }
    }
}

impl<DB: DatabaseTrait + ?Sized> UserTypeResource<DB> {
    pub fn new(database: Arc<Pool>) -> Self {
        Self {
            database,
            _marker: PhantomData
        }
    }
}

/* **************************************************************************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* **************************************************************************************************************** */

impl<DB: DatabaseTrait + ?Sized + 'static> YummyCacheResourceFactory for ResourceFactory<DB> {
    fn user_information(&self) -> Box<dyn YummyCacheResource<K=UserId, V=UserInformationModel>> {
        Box::new(UserInformationResource::<DB>::new(self.database.clone()))
    }

    fn user_metas(&self) -> Box<dyn YummyCacheResource<K=UserId, V=UserMetaCollection>> {
        Box::new(UserMetaResource::<DB>::new(self.database.clone()))
    }

    fn user_type(&self) -> Box<dyn YummyCacheResource<K=UserId, V=yummy_model::UserType>> {
        Box::new(UserTypeResource::<DB>::new(self.database.clone()))
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

impl<DB: DatabaseTrait + ?Sized> YummyCacheResource for UserMetaResource<DB> {
    type K=UserId;
    type V=UserMetaCollection;

    fn get(&self, key: &Self::K) -> anyhow::Result<Option<Self::V>> {
        let mut connection = self.database.get()?;
        let result = DB::get_user_meta(&mut connection, key, UserMetaAccess::System)?;
        Ok(Some(result))
    }
}

impl<DB: DatabaseTrait + ?Sized> YummyCacheResource for UserTypeResource<DB> {
    type K=UserId;
    type V=UserType;

    fn get(&self, key: &Self::K) -> anyhow::Result<Option<Self::V>> {
        let mut connection = self.database.get()?;
        let result = DB::get_user_type(&mut connection, key)?;
        Ok(Some(result))
    }
}

/* **************************************************************************************************************** */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */
