use actix::Addr;
use actix_web::{web::{Data, Json}, HttpResponse};
use database::DatabaseTrait;
use general::{error::YummyError, web::GenericAnswer, auth::ApiIntegration};
use manager::api::auth::{AuthManager, RefreshToken, DeviceIdAuth, EmailAuth, CustomIdAuth};
use validator::Validate;

use crate::websocket::request::{Request, AuthType};

macro_rules! as_error {
    ($error: expr) => {
        HttpResponse::Ok().json(GenericAnswer {
            status: false,
            result: Some($error.to_string()),
        })
    }
}

macro_rules! as_ok {
    ($result: expr) => {
        HttpResponse::Ok().json(GenericAnswer {
            status: true,
            result: Some($result),
        })
    }
}

macro_rules! as_response {
    ($auth_manager: expr, $message: expr) => {
        {
            match $message.validate() {
                Ok(_) => match $auth_manager.get_ref().send($message).await {
                    Ok(actix_result) => match actix_result {
                        Ok(result) => as_ok!(result),
                        Err(error) => as_error!(error)
                    },
                    Err(error) => as_error!(error)
                },
                Err(error) => as_error!(error)
            }
        }
    };
}

async fn auth<DB: DatabaseTrait + Unpin + 'static>(auth_type: AuthType, auth_manager: Data<Addr<AuthManager<DB>>>) -> HttpResponse {
    match auth_type {
        AuthType::Email { email, password, if_not_exist_create } => as_response!(auth_manager, EmailAuth { email: email.clone(), password: password.clone(), if_not_exist_create }),
        AuthType::DeviceId { id } => as_response!(auth_manager, DeviceIdAuth::new(id.clone())),
        AuthType::CustomId { id } => as_response!(auth_manager, CustomIdAuth::new(id.clone())),
        AuthType::Refresh { token } => as_response!(auth_manager, RefreshToken { token: token.clone() }),
    }
}

pub async fn http_query<DB: DatabaseTrait + Unpin + 'static>(auth_manager: Data<Addr<AuthManager<DB>>>, request: Result<Json<Request>, actix_web::Error>, _: ApiIntegration) -> Result<HttpResponse, YummyError> {
    let response = match request?.0 {
        Request::Auth { auth_type } => auth(auth_type, auth_manager).await,
    };

    Ok(response)
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
    use super::http_query;


    fn config(cfg: &mut web::ServiceConfig) {
        let config = ::general::config::get_configuration();
        let connection = create_connection(":memory:").unwrap();
        create_database(&mut connection.clone().get().unwrap()).unwrap();
        let auth_manager = Data::new(AuthManager::<database::SqliteStore>::new(config.clone(), Arc::new(connection)).start());

        let query_cfg = QueryConfig::default()
            .error_handler(|err, _| {
                log::error!("{:?}", err);
                InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
            });

        cfg.app_data(query_cfg)
            .app_data(JsonConfig::default().error_handler(json_error_handler))
            .app_data(Data::new(config))
            .app_data(auth_manager.clone())

            .route("/v1/query", web::post().to(http_query::<database::SqliteStore>));
    }

    #[actix_web::test]
    async fn empty_body() {
        let app = test::init_service(App::new().configure(config)).await;

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_COOKIE_KEY.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({}))
            .to_request();

        let res: Answer = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, false);
    }

    #[actix_web::test]
    async fn fail_auth_email_1() {
        let app = test::init_service(App::new().configure(config)).await;

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_COOKIE_KEY.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "Email",
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
    async fn auth_email_2() {
        let app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_COOKIE_KEY.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "Email",
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
        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_COOKIE_KEY.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "DeviceId",
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
        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_COOKIE_KEY.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "DeviceId",
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, false);
        assert!(res.result.is_some());
        assert!(!res.result.unwrap().is_empty());
    }

    #[actix_web::test]
    async fn refresh_token_1() {
        let app = test::init_service(App::new().configure(config)).await;

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_COOKIE_KEY.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "Email",
                "email": "erhanbaris@gmail.com",
                "password": "erhan",
                "create": true
            }))
            .to_request();

        let response: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_COOKIE_KEY.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "Refresh",
                "token": response.result.unwrap()
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);
        assert!(res.result.is_some());
        assert!(!res.result.unwrap().is_empty());
    }

    #[actix_web::test]
    async fn refresh_token_2() {
        let app = test::init_service(App::new().configure(config)).await;

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_COOKIE_KEY.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "DeviceId",
                "id": "1234567890"
            }))
            .to_request();

        let response: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_COOKIE_KEY.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "Refresh",
                "token": response.result.unwrap()
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);
        assert!(res.result.is_some());
        assert!(!res.result.unwrap().is_empty());
    }

    #[actix_web::test]
    async fn fail_auth_refresh_token_1() {
        let app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_COOKIE_KEY.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "Refresh"
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, false);
        assert!(res.result.is_some());
        assert!(!res.result.unwrap().is_empty());
    }

    #[actix_web::test]
    async fn fail_auth_refresh_token_2() {
        let app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_COOKIE_KEY.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "Refresh",
                "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2Njg1MDU5NzgsInVzZXIiOnsiaWQiOiJkYmZjNGQxMy1hZThmLTQzNDAtYWJjZi1kZDgyODA3MzExOGIiLCJzZXNzaW9uIjoiM2Y4NWY1ODMtZTY3OS00YTY4LTk0NTYtMmEwYzk4YzJiMzcwIiwibmFtZSI6bnVsbCwiZW1haWwiOm51bGx9fQ.8pfVfgOYOKMR_nOamiy2lNhpTUGF56cRqTIg1qVuCbI"
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, false);
        assert!(res.result.is_some());
        assert!(!res.result.unwrap().is_empty());
    }
}
