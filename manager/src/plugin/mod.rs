use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, collections::HashMap, rc::Rc, cell::RefCell};

use general::{model::UserType, meta::{MetaType, UserMetaAccess, MetaAction}, config::YummyConfig};

use crate::auth::model::{EmailAuthRequest, DeviceIdAuthRequest, CustomIdAuthRequest, LogoutRequest, RefreshTokenRequest, RestoreTokenRequest};

pub mod lua;

macro_rules! create_plugin_func {
    ($pre: ident, $post: ident, $model: path) => {
        fn $pre <'a>(&self, _model: Rc<RefCell<$model>>) -> anyhow::Result<()> { Ok(()) }
        fn $post <'a>(&self, _model: Rc<RefCell<$model>>, _successed: bool) -> anyhow::Result<()> { Ok(()) }
    }
}

macro_rules! create_executer_func {
    ($pre_func_name: ident, $post_func_name: ident, $model: path) => {
        pub fn $pre_func_name(&self, model: $model) -> anyhow::Result<$model> {
            let model = Rc::new(RefCell::new(model));
            for plugin in self.auth_interfaces.iter() {
                if plugin.active.load(Ordering::Relaxed) {
                    plugin.plugin.$pre_func_name(model.clone())?;
                }
            }
    
            match Rc::try_unwrap(model) {
                Ok(refcell) => {
                    Ok(refcell.into_inner())
                },
                Err(_) => Err(anyhow::anyhow!("'#func_name' function failed. 'model' object saved in lua and that is cause a memory leak."))
            }
        }


        pub fn $post_func_name(&self, model: $model, successed: bool) -> anyhow::Result<$model> {
            let model = Rc::new(RefCell::new(model));
            for plugin in self.auth_interfaces.iter() {
                if plugin.active.load(Ordering::Relaxed) {
                    plugin.plugin.$post_func_name(model.clone(), successed)?;
                }
            }
    
            match Rc::try_unwrap(model) {
                Ok(refcell) => {
                    Ok(refcell.into_inner())
                },
                Err(_) => Err(anyhow::anyhow!("'$post_func_name' function failed. 'model' object saved in lua and that is cause a memory leak."))
            }
        }
    }
}

pub trait YummyPlugin {
    create_plugin_func!(pre_email_auth, post_email_auth, EmailAuthRequest);
    create_plugin_func!(pre_deviceid_auth, post_deviceid_auth, DeviceIdAuthRequest);
    create_plugin_func!(pre_customid_auth, post_customid_auth, CustomIdAuthRequest);
    create_plugin_func!(pre_logout, post_logout, LogoutRequest);
    create_plugin_func!(pre_refresh_token, post_refresh_token, RefreshTokenRequest);
    create_plugin_func!(pre_restore_token, post_restore_token, RestoreTokenRequest);
}

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

/*pub trait UserProxy {
    fn get_user(&self, user: UserId) -> Option<UserInformationModel>;
    fn get_user_meta(&self, user: UserId, key: String) -> Option<MetaType<UserMetaAccess>>;
    fn set_user_meta(&self, user: UserId, key: String, meta: MetaType<UserMetaAccess>) -> Option<MetaType<UserMetaAccess>>;
    fn remove_user_meta(&self, user: UserId, key: String);
}*/

pub enum YummyAuthError {
    AuthFailed(String),
    Other(String)
}

pub struct PluginInfo {
    pub plugin: Box<dyn YummyPlugin>,
    pub name: String,
    pub active: AtomicBool
}

pub struct PluginExecuter {
    //user_manager: Box<dyn UserProxy>,
    auth_interfaces: Vec<PluginInfo>
}

pub trait YummyPluginInstaller {
    fn install(executer: PluginExecuter, config: Arc<YummyConfig>) -> PluginExecuter;
}

impl PluginExecuter {
    pub fn new() -> Self {
        Self {
            auth_interfaces: Vec::new()
        }
    }

    pub fn add_plugin(&mut self, name: String, plugin: Box<dyn YummyPlugin>) {
        self.auth_interfaces.push(PluginInfo {
            plugin,
            name,
            active: AtomicBool::new(true)
        });
    }

    create_executer_func!(pre_email_auth, post_email_auth, EmailAuthRequest);
    create_executer_func!(pre_deviceid_auth, post_deviceid_auth, DeviceIdAuthRequest);
    create_executer_func!(pre_customid_auth, post_customid_auth, CustomIdAuthRequest);
    create_executer_func!(pre_logout, post_logout, LogoutRequest);
    create_executer_func!(pre_refresh_token, post_refresh_token, RefreshTokenRequest);
    create_executer_func!(pre_restore_token, post_restore_token, RestoreTokenRequest);
}
