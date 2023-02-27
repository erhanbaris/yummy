use general::{meta::UserMetaAccess, model::{UserId, UserInformationModel}};

use crate::cache::YummyCacheResource;

pub trait StateResourceTrait {
    fn get_user_information(&self, user_id: &UserId, access_type: UserMetaAccess) -> anyhow::Result<Option<UserInformationModel>>;
}

pub trait YummyCacheResourceFactory {
    fn user_information(&self) -> Box<dyn YummyCacheResource<K=UserId, V=UserInformationModel>>;
}
