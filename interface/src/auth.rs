use std::sync::Arc;

use general::{auth::UserAuth, password::Password, client::ClientTrait};

pub struct YummyEmailAuthModel {
    pub ref_id: usize,
    pub auth: Arc<Option<UserAuth>>,
    pub email: String,
    pub password: Password,
    pub if_not_exist_create: bool,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

pub trait YummyAuthInterface {
    fn pre_email_auth(&self, model: YummyEmailAuthModel) -> anyhow::Result<YummyEmailAuthModel> { Ok(model) }
    fn post_email_auth(&self, model: YummyEmailAuthModel) -> anyhow::Result<YummyEmailAuthModel> { Ok(model) }
}

pub struct DummyYummyAuthPlugin;

impl YummyAuthInterface for DummyYummyAuthPlugin {
    fn post_email_auth(&self, model: YummyEmailAuthModel) -> anyhow::Result<YummyEmailAuthModel> {
        Ok(model)
    }
}
