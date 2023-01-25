use std::{sync::Arc, cell::RefCell, rc::Rc};

use general::{auth::UserAuth, password::Password, client::ClientTrait};

use crate::UserProxy;

pub struct YummyEmailAuthModel {
    pub ref_id: usize,
    pub auth: Arc<Option<UserAuth>>,
    pub email: String,
    pub password: Password,
    pub if_not_exist_create: bool,
    pub socket: Arc<dyn ClientTrait + Sync + Send>
}

pub trait YummyAuthInterface {
    fn pre_email_auth<'a>(&self, _user_manager: &'a dyn UserProxy, _model: Rc<RefCell<YummyEmailAuthModel>>) -> anyhow::Result<()> { Ok(()) }
    fn post_email_auth<'a>(&self, _user_manager: &'a dyn UserProxy, _model: Rc<RefCell<YummyEmailAuthModel>>) -> anyhow::Result<()> { Ok(()) }
}

#[derive(Default)]
pub struct DummyYummyAuthPlugin;

impl YummyAuthInterface for DummyYummyAuthPlugin {
    fn pre_email_auth<'a>(&self, _user_manager: &'a dyn UserProxy, _model: Rc<RefCell<YummyEmailAuthModel>>) -> anyhow::Result<()> {
        println!("pre email auth");
        Ok(())
    }

    fn post_email_auth<'a>(&self, _user_manager: &'a dyn UserProxy, _model: Rc<RefCell<YummyEmailAuthModel>>) -> anyhow::Result<()> {
        println!("post email auth");
        Ok(())
    }
}

#[derive(Default)]
pub struct DummyUserProxy;

impl UserProxy for DummyUserProxy {
    fn get_user(&self, _user: general::model::UserId) -> Option<database::model::UserInformationModel> {
        todo!()
    }

    fn get_user_meta(&self, _user: general::model::UserId, _key: String) -> Option<general::meta::MetaType<general::meta::UserMetaAccess>> {
        todo!()
    }

    fn set_user_meta(&self, _user: general::model::UserId, _key: String, _meta: general::meta::MetaType<general::meta::UserMetaAccess>) -> Option<general::meta::MetaType<general::meta::UserMetaAccess>> {
        todo!()
    }

    fn remove_user_meta(&self, _user: general::model::UserId, _key: String) {
        todo!()
    }
}