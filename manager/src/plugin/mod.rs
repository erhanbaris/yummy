/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* **************************************************************************************************************** */
// pub mod lua;
pub mod python;

/* **************************************************************************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, rc::Rc, cell::RefCell, marker::PhantomData};

use cache::state::YummyState;
use database::{DatabaseTrait, DefaultDatabaseStore};
use general::database::Pool;
use model::{config::YummyConfig, UserId, meta::{UserMetaAccess, MetaType}};

use crate::{auth::model::{EmailAuthRequest, DeviceIdAuthRequest, CustomIdAuthRequest, LogoutRequest, RefreshTokenRequest, RestoreTokenRequest, ConnUserDisconnect}, conn::model::UserConnected, user::{model::{GetUserInformation, UpdateUser}, UserLogic}, room::model::{CreateRoomRequest, UpdateRoom, JoinToRoomRequest, ProcessWaitingUser, KickUserFromRoom, DisconnectFromRoomRequest, MessageToRoomRequest, RoomListRequest, WaitingRoomJoins, GetRoomRequest}};

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* **************************************************** MACROS **************************************************** */
/* **************************************************************************************************************** */
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
            for plugin in self.plugins.iter() {
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
            for plugin in self.plugins.iter() {
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

/* **************************************************************************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
#[derive(Default)]
pub struct PluginBuilder {
    installers: Vec<Box<dyn YummyPluginInstaller>>
}

#[derive(Clone)]
pub struct YummyPluginContext<DB: DatabaseTrait + ?Sized + 'static> {
    user_logic: UserLogic<DB>,
    _marker: PhantomData<DB>
}

pub struct PluginInfo {
    pub plugin: Box<dyn YummyPlugin>,
    pub name: String,
    pub active: AtomicBool
}

pub struct PluginExecuter {
    plugins: Vec<PluginInfo>,
    pub context: YummyPluginContext<DefaultDatabaseStore>
}

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* **************************************************************************************************************** */
pub enum YummyAuthError {
    AuthFailed(String),
    Other(String)
}

/* **************************************************************************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* **************************************************************************************************************** */
pub trait YummyPluginInstaller {
    fn install(&self, executer: &mut PluginExecuter, config: Arc<YummyConfig>);
}

pub trait YummyPlugin {
    // Auth manager
    create_plugin_func!(pre_email_auth, post_email_auth, EmailAuthRequest);
    create_plugin_func!(pre_deviceid_auth, post_deviceid_auth, DeviceIdAuthRequest);
    create_plugin_func!(pre_customid_auth, post_customid_auth, CustomIdAuthRequest);
    create_plugin_func!(pre_logout, post_logout, LogoutRequest);
    create_plugin_func!(pre_refresh_token, post_refresh_token, RefreshTokenRequest);
    create_plugin_func!(pre_restore_token, post_restore_token, RestoreTokenRequest);

    // Connection manager
    create_plugin_func!(pre_user_connected, post_user_connected, UserConnected);
    create_plugin_func!(pre_user_disconnected, post_user_disconnected, ConnUserDisconnect);

    // User manager
    create_plugin_func!(pre_get_user_information, post_get_user_information, GetUserInformation);
    create_plugin_func!(pre_update_user, post_update_user, UpdateUser);

    // Room Manager
    create_plugin_func!(pre_create_room, post_create_room, CreateRoomRequest);
    create_plugin_func!(pre_update_room, post_update_room, UpdateRoom);
    create_plugin_func!(pre_join_to_room, post_join_to_room, JoinToRoomRequest);
    create_plugin_func!(pre_process_waiting_user, post_process_waiting_user, ProcessWaitingUser);
    create_plugin_func!(pre_kick_user_from_room, post_kick_user_from_room, KickUserFromRoom);
    create_plugin_func!(pre_disconnect_from_room_request, post_disconnect_from_room_request, DisconnectFromRoomRequest);
    create_plugin_func!(pre_message_to_room_request, post_message_to_room_request, MessageToRoomRequest);
    create_plugin_func!(pre_room_list_request, post_room_list_request, RoomListRequest);
    create_plugin_func!(pre_waiting_room_joins, post_waiting_room_joins, WaitingRoomJoins);
    create_plugin_func!(pre_get_room_request, post_get_room_request, GetRoomRequest);
}

/* **************************************************************************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* **************************************************************************************************************** */
impl PluginExecuter {
    pub fn new(config: Arc<YummyConfig>, states: YummyState, database: Arc<Pool>) -> Self {
        Self {
            plugins: Vec::new(),
            context: YummyPluginContext {
                user_logic: UserLogic::new(config, states, database),
                _marker: PhantomData
            }
        }
    }

    pub fn add_plugin(&mut self, name: String, plugin: Box<dyn YummyPlugin>) {
        self.plugins.push(PluginInfo {
            plugin,
            name,
            active: AtomicBool::new(true)
        });
    }

    // Auth manager
    create_executer_func!(pre_email_auth, post_email_auth, EmailAuthRequest);
    create_executer_func!(pre_deviceid_auth, post_deviceid_auth, DeviceIdAuthRequest);
    create_executer_func!(pre_customid_auth, post_customid_auth, CustomIdAuthRequest);
    create_executer_func!(pre_logout, post_logout, LogoutRequest);
    create_executer_func!(pre_refresh_token, post_refresh_token, RefreshTokenRequest);
    create_executer_func!(pre_restore_token, post_restore_token, RestoreTokenRequest);
    
    // Connection manager
    create_executer_func!(pre_user_connected, post_user_connected, UserConnected);
    create_executer_func!(pre_user_disconnected, post_user_disconnected, ConnUserDisconnect);

    // User manager
    create_executer_func!(pre_get_user_information, post_get_user_information, GetUserInformation);
    create_executer_func!(pre_update_user, post_update_user, UpdateUser);

    // Room Manager
    create_executer_func!(pre_create_room, post_create_room, CreateRoomRequest);
    create_executer_func!(pre_update_room, post_update_room, UpdateRoom);
    create_executer_func!(pre_join_to_room, post_join_to_room, JoinToRoomRequest);
    create_executer_func!(pre_process_waiting_user, post_process_waiting_user, ProcessWaitingUser);
    create_executer_func!(pre_kick_user_from_room, post_kick_user_from_room, KickUserFromRoom);
    create_executer_func!(pre_disconnect_from_room_request, post_disconnect_from_room_request, DisconnectFromRoomRequest);
    create_executer_func!(pre_message_to_room_request, post_message_to_room_request, MessageToRoomRequest);
    create_executer_func!(pre_room_list_request, post_room_list_request, RoomListRequest);
    create_executer_func!(pre_waiting_room_joins, post_waiting_room_joins, WaitingRoomJoins);
    create_executer_func!(pre_get_room_request, post_get_room_request, GetRoomRequest);
}

impl<DB: database::DatabaseTrait> YummyPluginContext<DB> {
    pub fn get_user_meta(&self, user_id: UserId, key: String) -> anyhow::Result<Option<MetaType<UserMetaAccess>>> {
        self.user_logic.get_user_meta(user_id, key)
    }
}

impl PluginBuilder {
    pub fn add_installer(&mut self, installer: Box<dyn YummyPluginInstaller>) {
        self.installers.push(installer);
    }

    pub fn build(&self, config: Arc<YummyConfig>, states: YummyState, database: Arc<Pool>) -> PluginExecuter {
        let mut executer = PluginExecuter::new(config.clone(), states, database); 
        for installer in self.installers.iter() {
            installer.install(&mut executer, config.clone());
        }

        executer
    }
}

/* **************************************************************************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */
