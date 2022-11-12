use core::{error::YummyError, web::GenericAnswer};

use actix::Addr;
use actix_web::{web::{Data, Json, ServiceConfig, post}, HttpResponse};
use database::auth::AuthStoreTrait;
use manager::api::auth::*;
use serde::Deserialize;
use secrecy::SecretString;


pub fn v1_scoped_config<A: AuthStoreTrait + std::marker::Unpin + 'static>(cfg: &mut ServiceConfig) {
    cfg.route("/authenticate/email", post().to(authenticate_email::<A>));
    cfg.route("/authenticate/deviceid", post().to(authenticate_deviceid::<A>));
    cfg.route("/authenticate/refresh", post().to(refresh_token::<A>));
}

#[derive(Debug, Deserialize)]
pub struct AuthenticateEmail {
    pub email: String,
    pub password: SecretString,

    #[serde(rename = "create")]
    #[serde(default)]
    pub if_not_exist_create: bool
}

#[derive(Debug, Deserialize)]
pub struct Refresh {
    pub token: String
}

#[derive(Debug, Deserialize)]
pub struct AuthenticateDeviceId {
    pub id: String
}

#[tracing::instrument(name="Authenticate device id", skip(auth_manager))]
async fn authenticate_deviceid<A: AuthStoreTrait + std::marker::Unpin + 'static>(auth_manager: Data<Addr<AuthManager<A>>>, auth_model: Result<Json<AuthenticateDeviceId>, actix_web::Error>) ->  Result<HttpResponse, YummyError> {
    let auth_model = auth_model.map_err(|e| YummyError::ActixError(e.into()))?.into_inner();
    
    let auth_result = auth_manager.get_ref().send(DeviceIdAuth(auth_model.id)).await.map_err(|_| YummyError::Unknown)??;

    Ok(HttpResponse::Ok().json(GenericAnswer {
        status: true,
        result: Some(auth_result),
    }))
}

#[tracing::instrument(name="Authenticate email", skip(auth_manager))]
async fn authenticate_email<A: AuthStoreTrait + std::marker::Unpin + 'static>(auth_manager: Data<Addr<AuthManager<A>>>, auth_model: Result<Json<AuthenticateEmail>, actix_web::Error>) ->  Result<HttpResponse, YummyError> {
    let auth_model = auth_model.map_err(|e| YummyError::ActixError(e.into()))?.into_inner();
    
    let auth_result = auth_manager.get_ref().send(EmailAuth {
        email: auth_model.email,
        password: auth_model.password,
        if_not_exist_create: auth_model.if_not_exist_create
    }).await.map_err(|_| YummyError::Unknown)??;

    Ok(HttpResponse::Ok().json(GenericAnswer {
        status: true,
        result: Some(auth_result),
    }))
}

#[tracing::instrument(name="Refresh token", skip(auth_manager))]
async fn refresh_token<A: AuthStoreTrait + std::marker::Unpin + 'static>(auth_manager: Data<Addr<AuthManager<A>>>, token: Result<Json<Refresh>, actix_web::Error>) ->  Result<HttpResponse, YummyError> {
    let token = token.map_err(|e| YummyError::ActixError(e.into()))?.into_inner();
    
    let auth_result = auth_manager.get_ref().send(RefreshToken(token.token)).await.map_err(|_| YummyError::Unknown)??;

    Ok(HttpResponse::Ok().json(GenericAnswer {
        status: true,
        result: Some(auth_result),
    }))
}
