use general::model::{UserId, UserInformationModel};

use crate::{cache::YummyCacheResource, state_resource::UserMetaInformation};

pub trait YummyCacheResourceFactory {
    fn user_information(&self) -> Box<dyn YummyCacheResource<K=UserId, V=UserInformationModel>>;
    fn user_metas(&self) -> Box<dyn YummyCacheResource<K=UserId, V=Vec<UserMetaInformation>>>;
}
