use crate::meta::UserMetaAccess;
use crate::model::{UserId, UserInformationModel};

pub trait StateResourceTrait {
    fn get_user_information(&self, user_id: &UserId, access_type: UserMetaAccess) -> anyhow::Result<Option<UserInformationModel>>;
}
