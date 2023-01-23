use auth::{YummyAuthInterface, YummyEmailAuthModel};

pub mod auth;

pub enum YummyAuthError {

}

pub struct PluginInfo {
    pub plugin: Box<dyn YummyAuthInterface>,
    pub name: String,
    pub active: bool
}

#[derive(Default)]
pub struct PluginExecuter {
    pub auth_interfaces: Vec<PluginInfo>
}

impl PluginExecuter {
    pub fn pre_email_auth(&self, model: YummyEmailAuthModel) -> anyhow::Result<YummyEmailAuthModel> {
        let mut model = model;
        for plugin in self.auth_interfaces.iter() {
            if plugin.active {
                model = plugin.plugin.pre_email_auth(model)?;
            }
        }

        Ok(model)
    }

    pub fn post_email_auth(&self, model: YummyEmailAuthModel) -> anyhow::Result<YummyEmailAuthModel> {
        let mut model = model;
        for plugin in self.auth_interfaces.iter() {
            if plugin.active {
                model = plugin.plugin.post_email_auth(model)?;
            }
        }

        Ok(model)
    }
}
