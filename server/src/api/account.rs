use core::error::YummyError;

use actix::Addr;
use actix_web::{HttpRequest, web::{Data, Json, Query, ServiceConfig, post}, HttpResponse};
use manager::{GameManager, api::auth::*};
use serde::Deserialize;

use crate::core::GenericAnswer;

pub fn v1_scoped_config(cfg: &mut ServiceConfig) {
    cfg.route("/authenticate/email", post().to(authenticate_email));
}

#[derive(Debug, Deserialize, Default)]
pub struct AuthenticateEmail {
    pub email: String,
    pub password: String,

    #[serde(rename = "create")]
    pub if_not_exist_create: bool
}

async fn authenticate_email(req: HttpRequest, manager: Data<Addr<GameManager>>, data: Json<AuthenticateEmail>, auth_model: Result<Query<AuthenticateEmail>, actix_web::Error>) ->  Result<actix_web::HttpResponse, YummyError> {
    let auth_model = match auth_model {
        Ok(auth_model) => auth_model.into_inner(),
        Err(error) => return Err(YummyError::WebsocketConnectArgument(error.to_string()))
    };
    
    let auth_result = manager.get_ref().send(EmailAuth {
        email: auth_model.email,
        password: auth_model.password,
        if_not_exist_create: auth_model.if_not_exist_create
    }).await;


    match auth_result {
        Ok(result) => match result {
            Ok(result) => Ok(HttpResponse::Ok().json(GenericAnswer {
                status: true,
                result: Some(result),
            })),
            Err(error) => Err(error)
        },
        Err(_) => Err(YummyError::Unknown)
    }
}
