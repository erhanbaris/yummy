use core::{error::YummyError, web::GenericAnswer};
use std::default;

use actix::Addr;
use actix_web::{HttpRequest, web::{Data, Json, Query, ServiceConfig, get}, HttpResponse, Responder};
use manager::{GameManager, api::auth::*};
use serde::Deserialize;
use secrecy::{SecretString, Secret};
use tracing::{warn, event};


pub fn v1_scoped_config(cfg: &mut ServiceConfig) {
    cfg.route("/authenticate/email", get().to(authenticate_email));
}

#[derive(Debug, Deserialize)]
pub struct AuthenticateEmail {
    pub email: String,
    pub password: SecretString,

    #[serde(rename = "create")]
    pub if_not_exist_create: bool
}

#[tracing::instrument(name="Authenticate email", skip(manager))]
async fn authenticate_email(manager: Data<Addr<GameManager>>, auth_model: Result<Query<AuthenticateEmail>, actix_web::Error>) ->  Result<HttpResponse, YummyError> {
    let auth_model = auth_model.map_err(|e| YummyError::ActixError(e.into()))?.into_inner();
    
    let auth_result = manager.get_ref().send(EmailAuth {
        email: auth_model.email,
        password: auth_model.password,
        if_not_exist_create: auth_model.if_not_exist_create
    }).await.map_err(|_| YummyError::Unknown)??;

    Ok(HttpResponse::Ok().json(GenericAnswer {
        status: true,
        result: Some(auth_result),
    }))
}
