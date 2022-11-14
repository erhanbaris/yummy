use general::{error::YummyError, web::GenericAnswer};

use actix::Addr;
use actix_web::{web::{Data, Json, ServiceConfig, post}, HttpResponse};
use database::auth::AuthStoreTrait;
use manager::api::auth::*;
use serde::Deserialize;

use crate::websocket::request::Request;

pub fn v1_scoped_config<A: AuthStoreTrait + std::marker::Unpin + 'static>(cfg: &mut ServiceConfig) {
    cfg.route("/authenticate/email", post().to(authenticate_email::<A>));
    cfg.route("/authenticate/deviceid", post().to(authenticate_deviceid::<A>));
    cfg.route("/authenticate/refresh", post().to(refresh_token::<A>));
    cfg.route("/query", post().to(query::<A>));
}

#[derive(Debug, Deserialize)]
pub struct AuthenticateEmail {
    pub email: String,
    pub password: String,

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
    
    let auth_result = auth_manager.get_ref().send(DeviceIdAuth::new(auth_model.id)).await.map_err(|_| YummyError::Unknown)??;

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

async fn query<A: AuthStoreTrait + Unpin + 'static>(_: Data<Addr<AuthManager<A>>>, request: Result<Json<Request>, actix_web::Error>) ->  Result<HttpResponse, YummyError> {
    log::info!("{:?}", request);
    Ok(HttpResponse::Ok().json(GenericAnswer {
        status: true,
        result: Some("It is alive"),
    }))
}

#[cfg(test)]
pub mod tests {
    use actix::Actor;
    use actix_web::test;
    use actix_web::HttpResponse;
    use actix_web::error::InternalError;
    use actix_web::web::{QueryConfig, JsonConfig};
    use actix_web::{web, web::Data, App};
    use database::{create_database, create_connection};
    use general::web::Answer;
    use general::web::GenericAnswer;
    use manager::api::auth::AuthManager;
    use serde_json::json;
    use std::sync::Arc;

    use crate::json_error_handler;


    fn config(cfg: &mut web::ServiceConfig) {
        let config = ::general::config::get_configuration();
        let connection = create_connection(":memory:").unwrap();
        create_database(&mut connection.clone().get().unwrap()).unwrap();
        let auth_manager = Data::new(AuthManager::<database::auth::AuthStore>::new(config.clone(), Arc::new(connection)).start());

        let query_cfg = QueryConfig::default()
            .error_handler(|err, _| {
                log::error!("{:?}", err);
                InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
            });

        cfg.app_data(query_cfg)
            .app_data(JsonConfig::default().error_handler(json_error_handler))
            .app_data(Data::new(config))
            .app_data(auth_manager.clone())

            .service(web::scope("/v1/account").configure(super::v1_scoped_config::<database::auth::AuthStore>));
    }

    #[actix_web::test]
    async fn fail_auth_email_1() {
        let app = test::init_service(App::new().configure(config)).await;

        let req = test::TestRequest::post().uri("/v1/account/authenticate/email")
            .set_json(json!({}))
            .to_request();

        let res: Answer = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, false);
    }

    #[actix_web::test]
    async fn fail_auth_email_2() {
        let app = test::init_service(App::new().configure(config)).await;

        let req = test::TestRequest::post().uri("/v1/account/authenticate/email")
            .set_json(json!({
                "email": "erhanbaris@gmail.com",
                "password": "erhan",
                "create": false
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, false);
        assert!(res.result.is_some());
        assert_eq!(res.result.unwrap(), "Email and/or password not valid".to_string());
    }

    #[actix_web::test]
    async fn auth_email_3() {
        let app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::post().uri("/v1/account/authenticate/email")
            .set_json(json!({
                "email": "erhanbaris@gmail.com",
                "password": "erhan",
                "create": true
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);
        assert!(res.result.is_some());
        assert!(!res.result.unwrap().is_empty());
    }

    #[actix_web::test]
    async fn auth_device_id() {
        let app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::post().uri("/v1/account/authenticate/deviceid")
            .set_json(json!({
                "id": "1234567890"
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);
        assert!(res.result.is_some());
        assert!(!res.result.unwrap().is_empty());
    }

    #[actix_web::test]
    async fn fail_auth_device_id() {
        let app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::post().uri("/v1/account/authenticate/deviceid")
            .set_json(json!({
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, false);
        assert!(res.result.is_some());
        assert!(!res.result.unwrap().is_empty());
    }

    #[actix_web::test]
    async fn refresh_token_2() {
        let app = test::init_service(App::new().configure(config)).await;

        let req = test::TestRequest::post().uri("/v1/account/authenticate/email")
            .set_json(json!({
                "email": "erhanbaris@gmail.com",
                "password": "erhan",
                "create": true
            }))
            .to_request();

        let response: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;

        let req = test::TestRequest::post().uri("/v1/account/authenticate/refresh")
            .set_json(json!({
                "token": response.result.unwrap()
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);
        assert!(res.result.is_some());
        assert!(!res.result.unwrap().is_empty());
    }

    #[actix_web::test]
    async fn refresh_token_1() {
        let app = test::init_service(App::new().configure(config)).await;

        let req = test::TestRequest::post().uri("/v1/account/authenticate/deviceid")
        .set_json(json!({
            "id": "1234567890"
        }))
        .to_request();

        let response: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;

        let req = test::TestRequest::post().uri("/v1/account/authenticate/refresh")
            .set_json(json!({
                "token": response.result.unwrap()
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);
        assert!(res.result.is_some());
        assert!(!res.result.unwrap().is_empty());
    }

    #[actix_web::test]
    async fn fail_auth_refresh_token() {
        let app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::post().uri("/v1/account/authenticate/refresh")
            .set_json(json!({
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, false);
        assert!(res.result.is_some());
        assert!(!res.result.unwrap().is_empty());
    }
}
