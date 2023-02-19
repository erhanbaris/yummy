pub mod model;

#[cfg(test)]
mod test;
mod logic;

use std::marker::PhantomData;
use std::sync::Arc;

use actix::{Context, Actor, Handler};
use database::{Pool, DatabaseTrait};

use general::config::YummyConfig;
use general::state::YummyState;
use general::web::GenericAnswer;

use crate::plugin::PluginExecuter;

use self::logic::UserLogic;
use self::model::*;

pub struct UserManager<DB: DatabaseTrait + ?Sized> {
    _config: Arc<YummyConfig>,
    _database: Arc<Pool>,
    _states: YummyState,
    executer: Arc<PluginExecuter>,
    _marker: PhantomData<DB>,
    logic: UserLogic<DB>
}

impl<DB: DatabaseTrait + ?Sized> UserManager<DB> {
    pub fn new(config: Arc<YummyConfig>, states: YummyState, database: Arc<Pool>, executer: Arc<PluginExecuter>) -> Self {
        Self {
            _config: config.clone(),
            _database: database.clone(),
            _states: states.clone(),
            executer,
            _marker: PhantomData,
            logic: UserLogic::new(config.clone(), states.clone(), database.clone())
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Actor for UserManager<DB> {
    type Context = Context<Self>;
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<GetUserInformation> for UserManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="GetUserInformation", skip(self, _ctx))]
    #[macros::plugin_api(name="get_user_information")]
    fn handle(&mut self, model: GetUserInformation, _ctx: &mut Context<Self>) -> Self::Result {
        let user = self.logic.get_user_information(&model)?;
        model.socket.send(GenericAnswer::success(model.request_id.clone(), UserResponse::UserInfo { user }).into());
        Ok(())
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<UpdateUser> for UserManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="UpdateUser", skip(self, _ctx))]
    #[macros::plugin_api(name="update_user")]
    fn handle(&mut self, model: UpdateUser, _ctx: &mut Context<Self>) -> Self::Result {
        let user = self.logic.update_user(&model)?;
        model.socket.send(user.into());
        Ok(())
    }
}
