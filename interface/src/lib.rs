use std::sync::atomic::{AtomicBool, Ordering};

use auth::{YummyAuthInterface, YummyEmailAuthModel};

pub mod auth;

pub enum YummyAuthError {

}

pub struct PluginInfo {
    pub plugin: Box<dyn YummyAuthInterface>,
    pub name: String,
    pub active: AtomicBool
}

#[derive(Default)]
pub struct PluginExecuter {
    pub auth_interfaces: Vec<PluginInfo>
}

impl PluginExecuter {
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
                model = plugin.plugin.pre_email_auth(model)?;
            }
        }

        Ok(model)
    }

    pub fn post_email_auth(&self, model: YummyEmailAuthModel) -> anyhow::Result<YummyEmailAuthModel> {
        let mut model = model;
        for plugin in self.auth_interfaces.iter() {
            if plugin.active.load(Ordering::Relaxed) {
                model = plugin.plugin.post_email_auth(model)?;
            }
        }

        Ok(model)
    }
}
