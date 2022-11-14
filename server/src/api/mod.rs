pub(crate) mod account;


use actix::Addr;
use actix_web::{web::{Data, Json}, HttpResponse};
use database::auth::AuthStoreTrait;
use general::{error::YummyError, web::GenericAnswer};
use manager::api::auth::{AuthManager, RefreshToken, DeviceIdAuth, EmailAuth};
use validator::Validate;

use crate::websocket::request::{Request, AuthType};

macro_rules! message_validate {
    ($model: expr) => {
        match $model.validate() {
            Ok(_) => $model,
            Err(e) => return Err(anyhow::anyhow!(e))
        }
    }
}

macro_rules! as_response {
    ($auth_manager: expr, $message: expr) => {
        Ok(HttpResponse::Ok().json(GenericAnswer {
            status: true,
            result: Some($auth_manager.get_ref().send(message_validate!($message)).await??),
        }))
    };
}

async fn auth<A: AuthStoreTrait + Unpin + 'static>(auth_type: AuthType, auth_manager: Data<Addr<AuthManager<A>>>) -> anyhow::Result<HttpResponse> {
    match auth_type {
        AuthType::Email { email, password, if_not_exist_create } => as_response!(auth_manager, EmailAuth { email: email.clone(), password: password.clone(), if_not_exist_create }),
        AuthType::DeviceId { id } => as_response!(auth_manager, DeviceIdAuth::new(id.clone())),
        AuthType::Refresh { token } => as_response!(auth_manager, RefreshToken { token: token.clone() }),
    }
}

pub async fn http_query<A: AuthStoreTrait + Unpin + 'static>(auth_manager: Data<Addr<AuthManager<A>>>, request: Result<Json<Request>, actix_web::Error>) ->  Result<HttpResponse, YummyError> {
    log::info!("{:?}", request);

    let result = match request.unwrap().0 {
        Request::Auth { auth_type } => auth(auth_type, auth_manager).await,
    };

    match result {
        Ok(result) => Ok(result),
        Err(error) => Ok(HttpResponse::Ok().json(GenericAnswer {
            status: true,
            result: Some(error.to_string()),
        }))
    }
}