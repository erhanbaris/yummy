pub mod model;

#[cfg(test)]
mod test;
mod logic;

use std::borrow::Cow;
use std::marker::PhantomData;
use std::sync::Arc;

use actix::{Context, Actor, Handler};
use yummy_database::DatabaseTrait;

use yummy_model::config::YummyConfig;
use yummy_model::web::GenericAnswer;
use yummy_general::database::Pool;
use yummy_cache::state::YummyState;
use crate::YummyModel;
use crate::plugin::PluginExecuter;

pub use self::logic::UserLogic;
use self::model::*;

pub struct UserManager<DB: DatabaseTrait + ?Sized> {
    executer: Arc<PluginExecuter>,
    _marker: PhantomData<DB>,
    logic: UserLogic<DB>
}

impl<DB: DatabaseTrait + ?Sized> UserManager<DB> {
    pub fn new(config: Arc<YummyConfig>, states: YummyState, database: Arc<Pool>, executer: Arc<PluginExecuter>) -> Self {
        Self {
            executer,
            _marker: PhantomData,
            logic: UserLogic::new(config, states, database)
        }
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Actor for UserManager<DB> {
    type Context = Context<Self>;
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<GetUserInformation> for UserManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="GetUserInformation", skip(self, _ctx))]
    #[yummy_macros::plugin_api(name="get_user_information", model="GetUserInformation")]
    fn handle(&mut self, model: GetUserInformation, _ctx: &mut Context<Self>) -> Self::Result {
        let user = self.logic.get_user_information(&model)?;
        model.socket.send(GenericAnswer::success(model.request_id, Cow::Borrowed(GetUserInformation::get_request_type()), UserResponse::UserInfo { user }).into());
        Ok(())
    }
}

impl<DB: DatabaseTrait + ?Sized + std::marker::Unpin + 'static> Handler<UpdateUser> for UserManager<DB> {
    type Result = anyhow::Result<()>;

    #[tracing::instrument(name="UpdateUser", skip(self, _ctx))]
    #[yummy_macros::plugin_api(name="update_user", model="UpdateUser")]
    fn handle(&mut self, model: UpdateUser, _ctx: &mut Context<Self>) -> Self::Result {
        let user = self.logic.update_user(&model)?;
        model.socket.send(user.into());
        Ok(())
    }
}
