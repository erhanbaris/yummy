use std::{sync::atomic::{AtomicBool, Ordering}, collections::HashMap, ops::Deref};

use auth::{YummyAuthInterface, YummyEmailAuthModel};
use general::{model::{UserId, UserType}, meta::{MetaType, UserMetaAccess, MetaAction}};
use database::model::UserInformationModel;

pub mod auth;

pub struct UpdateUser {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub device_id: Option<String>,
    pub custom_id: Option<String>,
    pub user_type: Option<UserType>,
    pub meta: Option<HashMap<String, MetaType<UserMetaAccess>>>,
    pub meta_action: Option<MetaAction>
}

pub trait UserProxy {
    fn get_user(&self, user: UserId) -> Option<UserInformationModel>;
    fn get_user_meta(&self, user: UserId, key: String) -> Option<MetaType<UserMetaAccess>>;
    fn set_user_meta(&self, user: UserId, key: String, meta: MetaType<UserMetaAccess>) -> Option<MetaType<UserMetaAccess>>;
    fn remove_user_meta(&self, user: UserId, key: String);
}

pub enum YummyAuthError {
    AuthFailed(String),
    Other(String)
}

pub struct PluginInfo {
    pub plugin: Box<dyn YummyAuthInterface>,
    pub name: String,
    pub active: AtomicBool
}

pub struct PluginExecuter {
    user_manager: Box<dyn UserProxy>,
    auth_interfaces: Vec<PluginInfo>
}

impl PluginExecuter {
    pub fn new(user_manager: Box<dyn UserProxy>) -> Self {
        Self {
            user_manager,
            auth_interfaces: Vec::new()
        }
    }

    pub fn add_auth_plugin(&mut self, name: String, plugin: Box<dyn YummyAuthInterface>) {
        self.auth_interfaces.push(PluginInfo {
            plugin,
            name,
            active: AtomicBool::new(true)
        });
    }

    pub fn pre_email_auth(&self, model: YummyEmailAuthModel) -> anyhow::Result<YummyEmailAuthModel> {
        let mut model = model;
        
        for plugin in self.auth_interfaces.iter() {
            if plugin.active.load(Ordering::Relaxed) {
                plugin.plugin.pre_email_auth(self.user_manager.deref(), &mut model)?;
            }
        }

        Ok(model)
    }

    pub fn post_email_auth(&self, model: YummyEmailAuthModel) -> anyhow::Result<YummyEmailAuthModel> {
        let mut model = model;
        for plugin in self.auth_interfaces.iter() {
            if plugin.active.load(Ordering::Relaxed) {
                plugin.plugin.post_email_auth(self.user_manager.deref(), &mut model)?;
            }
        }

        Ok(model)
    }
}
