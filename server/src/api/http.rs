use std::sync::Arc;

use actix::Addr;
use actix_web::{HttpResponse, HttpRequest};
use actix_web::{web::{Data, Json}};
use database::DatabaseTrait;
use general::web::{GenericAnswer, Answer};
use general::{auth::{UserAuth, ApiIntegration}, error::YummyError};
use manager::room::RoomManager;
use manager::{auth::AuthManager, user::UserManager};
use manager::response::Response;

use crate::api::request::Request;
use super::{process_auth, process_user, process_room};

#[tracing::instrument(name="http_query", skip(req, auth_manager, user_manager, _integration))]
pub async fn http_query<DB: DatabaseTrait + Unpin + 'static>(req: HttpRequest, auth_manager: Data<Addr<AuthManager<DB>>>, user_manager: Data<Addr<UserManager<DB>>>, room_manager: Data<Addr<RoomManager<DB>>>, request: Result<Json<Request>, actix_web::Error>, _integration: ApiIntegration) -> Result<HttpResponse, YummyError> {
    let user = Arc::new(UserAuth::parse(&req));
    
    let response = match request?.0 {
        Request::Auth { auth_type } => process_auth(auth_type, auth_manager.as_ref().clone(), user.clone(), None).await,
        Request::User { user_type } => process_user(user_type, user_manager.as_ref().clone(), user.clone()).await,
        Request::Room { room_type } => process_room(room_type, room_manager.as_ref().clone(), user.clone()).await,
    };

    match response {
        Ok(response) => {
            let message = match response {
                Response::Auth(token, _) => HttpResponse::Ok().json(GenericAnswer::success(token)),
                Response::UserInformation(model) => HttpResponse::Ok().json(GenericAnswer::success(model)),
                Response::RoomInformation(room_id) => HttpResponse::Ok().json(GenericAnswer::success(room_id)),
                Response::None => HttpResponse::Ok().json(Answer::success()),
            };

            Ok(message)
        },
        Err(error) => Ok(HttpResponse::Ok().json(GenericAnswer::fail(error.to_string())))
    }
}

#[cfg(test)]
pub mod tests {
    use actix::Actor;
    use actix_web::test;
    use actix_web::HttpResponse;
    use actix_web::error::InternalError;
    use actix_web::web::{QueryConfig, JsonConfig};
    use actix_web::{web, web::Data, App};
    use database::model::UserInformationModel;
    use database::{create_database, create_connection};
    use general::state::YummyState;
    use general::web::Answer;
    use general::web::GenericAnswer;
    use manager::auth::AuthManager;
    use manager::room::RoomManager;
    use manager::user::UserManager;
    use serde_json::json;
    use uuid::Uuid;
    use std::env::temp_dir;
    use std::sync::Arc;

    use crate::json_error_handler;
    use super::http_query;

    #[cfg(feature = "stateless")]
    use general::test::cleanup_redis;

    fn config(cfg: &mut web::ServiceConfig) {
        let mut db_location = temp_dir();
        db_location.push(format!("{}.db", Uuid::new_v4()));

        ::general::config::configure_environment();
        let config = ::general::config::get_configuration();
        let connection = create_connection(db_location.to_str().unwrap()).unwrap();
        create_database(&mut connection.clone().get().unwrap()).unwrap();

        #[cfg(feature = "stateless")]
        let conn = r2d2::Pool::new(redis::Client::open(config.redis_url.clone()).unwrap()).unwrap();


        let states = YummyState::new(#[cfg(feature = "stateless")] conn);
        let auth_manager = Data::new(AuthManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone())).start());
        let user_manager = Data::new(UserManager::<database::SqliteStore>::new(config.clone(), states.clone(), Arc::new(connection.clone())).start());
        let room_manager = Data::new(RoomManager::<database::SqliteStore>::new(config.clone(), states, Arc::new(connection)).start());

        let query_cfg = QueryConfig::default()
            .error_handler(|err, _| {
                log::error!("{:?}", err);
                InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
            });

        cfg.app_data(query_cfg)
            .app_data(JsonConfig::default().error_handler(json_error_handler))
            .app_data(Data::new(config))
            .app_data(auth_manager.clone())
            .app_data(user_manager.clone())
            .app_data(room_manager.clone())

            .route("/v1/query", web::post().to(http_query::<database::SqliteStore>));
    }

    #[actix_web::test]
    async fn empty_body() {
        let app = test::init_service(App::new().configure(config)).await;

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({}))
            .to_request();

        let res: Answer = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, false);
    }

    #[actix_web::test]
    async fn fail_auth_email_1() {
        let app = test::init_service(App::new().configure(config)).await;

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
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
        assert_eq!(res.result, "Email and/or password not valid".to_string());
    }

    #[actix_web::test]
    async fn auth_email_2() {
        let app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
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
        assert!(!res.result.is_empty());
    }

    #[actix_web::test]
    async fn auth_device_id() {
        let app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "DeviceId",
                "id": "1234567890"
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);
        assert!(!res.result.is_empty());
    }

    #[actix_web::test]
    async fn fail_auth_device_id() {
        let app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "DeviceId",
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, false);
        assert!(!res.result.is_empty());
    }

    #[actix_web::test]
    async fn refresh_token_1() {
        let app = test::init_service(App::new().configure(config)).await;

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
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
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "Refresh",
                "token": response.result
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);
        assert!(!res.result.is_empty());
    }

    #[actix_web::test]
    async fn refresh_token_2() {
        let app = test::init_service(App::new().configure(config)).await;

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "DeviceId",
                "id": "1234567890"
            }))
            .to_request();

        let response: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "Refresh",
                "token": response.result
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);
        assert!(!res.result.is_empty());
    }

    #[actix_web::test]
    async fn fail_auth_refresh_token_1() {
        let app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "Refresh"
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, false);
        assert!(!res.result.is_empty());
    }

    #[actix_web::test]
    async fn fail_auth_refresh_token_2() {
        let app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "Refresh",
                "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2Njg1MDU5NzgsInVzZXIiOnsiaWQiOiJkYmZjNGQxMy1hZThmLTQzNDAtYWJjZi1kZDgyODA3MzExOGIiLCJzZXNzaW9uIjoiM2Y4NWY1ODMtZTY3OS00YTY4LTk0NTYtMmEwYzk4YzJiMzcwIiwibmFtZSI6bnVsbCwiZW1haWwiOm51bGx9fQ.8pfVfgOYOKMR_nOamiy2lNhpTUGF56cRqTIg1qVuCbI"
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, false);
        assert!(!res.result.is_empty());
    }

    #[actix_web::test]
    async fn me() {
        let app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "DeviceId",
                "id": "1234567890"
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);

        let token = res.result.as_ref().unwrap();
        assert!(!token.is_empty());

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .append_header((general::config::DEFAULT_USER_AUTH_KEY_NAME.to_string(), token.to_string()))
            .set_json(json!({
                "type": "User",
                "user_type": "Me"
            }))
            .to_request();

        let res: GenericAnswer<serde_json::Value> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);
    }

    #[actix_web::test]
    async fn fail_me() {
        let app = test::init_service(App::new().configure(config)).await;
        
        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "User",
                "user_type": "Me"
            }))
            .to_request();

        let res: GenericAnswer<serde_json::Value> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, false);
    }

    #[actix_web::test]
    async fn get_private_user() {
        let app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "DeviceId",
                "id": "1234567890"
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);

        let token = res.result.as_ref().unwrap();
        assert!(!token.is_empty());

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .append_header((general::config::DEFAULT_USER_AUTH_KEY_NAME.to_string(), token.to_string()))
            .set_json(json!({
                "type": "User",
                "user_type": "Me"
            }))
            .to_request();

        let res: GenericAnswer<UserInformationModel> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);
        let original_user_model = res.result;

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .append_header((general::config::DEFAULT_USER_AUTH_KEY_NAME.to_string(), token.to_string()))
            .set_json(json!({
                "type": "User",
                "user_type": "Get",
                "user": original_user_model.id.to_string()
            }))
            .to_request();

        let res: GenericAnswer<UserInformationModel> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);
        let fetched_user_model = res.result;

        assert_eq!(original_user_model.id, fetched_user_model.id);
    
    }

    #[actix_web::test]
    async fn fail_get_private_user() {
        let app = test::init_service(App::new().configure(config)).await;
        
        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "User",
                "user_type": "Me"
            }))
            .to_request();

        let res: GenericAnswer<serde_json::Value> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, false);
    }

    #[actix_web::test]
    async fn fail_update_user_1() {
        let app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "DeviceId",
                "id": "1234567890"
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);

        let token = res.result.as_ref().unwrap();
        assert!(!token.is_empty());

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .append_header((general::config::DEFAULT_USER_AUTH_KEY_NAME.to_string(), token.to_string()))
            .set_json(json!({
                "type": "User",
                "user_type": "Update",
                "updates": []
            }))
            .to_request();

        let res: Answer = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, false);
    }

    #[actix_web::test]
    async fn fail_update_user_2() {
        let app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "DeviceId",
                "id": "1234567890"
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);

        let token = res.result.as_ref().unwrap();
        assert!(!token.is_empty());

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .append_header((general::config::DEFAULT_USER_AUTH_KEY_NAME.to_string(), token.to_string()))
            .set_json(json!({
                "type": "User",
                "user_type": "Update",
                "updates": [{}]
            }))
            .to_request();

        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, false);
    }


    #[actix_web::test]
    async fn update_user_1() {
        let app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
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

        let token = res.result.as_ref().unwrap();
        assert!(!token.is_empty());

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .append_header((general::config::DEFAULT_USER_AUTH_KEY_NAME.to_string(), token.to_string()))
            .set_json(json!({
                "type": "User",
                "user_type": "Update",
                "password": "baris"
            }))
            .to_request();

        let res: Answer = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .append_header((general::config::DEFAULT_USER_AUTH_KEY_NAME.to_string(), token.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "Logout"
            }))
            .to_request();
        let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .set_json(json!({
                "type": "Auth",
                "auth_type": "Email",
                "email": "erhanbaris@gmail.com",
                "password": "baris",
                "create": false
            }))
            .to_request();

    let res: GenericAnswer<String> = test::call_and_read_body_json(&app, req).await;
    assert_eq!(res.status, true);
    }


    #[actix_web::test]
    async fn update_user_2() {
        let app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
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

        let token = res.result.as_ref().unwrap();
        assert!(!token.is_empty());

        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .append_header((general::config::DEFAULT_USER_AUTH_KEY_NAME.to_string(), token.to_string()))
            .set_json(json!({
                "type": "User",
                "user_type": "Update",
                "name": "Erhan BARIS",
                "custom_id": "1234567890",
                "device_id": "987654321"
            }))
            .to_request();

        let res: Answer = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);

        let req = test::TestRequest::post().uri("/v1/query")
        .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
        .append_header((general::config::DEFAULT_USER_AUTH_KEY_NAME.to_string(), token.to_string()))
        .set_json(json!({
            "type": "User",
            "user_type": "Me"
        }))
        .to_request();

        let res: GenericAnswer<UserInformationModel> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);

        let me = res.result;
        assert_eq!(me.custom_id, Some("1234567890".to_string()));
        assert_eq!(me.device_id, Some("987654321".to_string()));
        assert_eq!(me.name, Some("Erhan BARIS".to_string()));
        assert_eq!(me.email, Some("erhanbaris@gmail.com".to_string()));
        
        /* Cleanup fields */        
        let req = test::TestRequest::post().uri("/v1/query")
            .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
            .append_header((general::config::DEFAULT_USER_AUTH_KEY_NAME.to_string(), token.to_string()))
            .set_json(json!({
                "type": "User",
                "user_type": "Update",
                "custom_id": "",
                "device_id": ""
            }))
            .to_request();

        let res: Answer = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);

        let req = test::TestRequest::post().uri("/v1/query")
        .append_header((general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()))
        .append_header((general::config::DEFAULT_USER_AUTH_KEY_NAME.to_string(), token.to_string()))
        .set_json(json!({
            "type": "User",
            "user_type": "Me"
        }))
        .to_request();

        let res: GenericAnswer<UserInformationModel> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.status, true);

        let me = res.result;
        assert_eq!(me.custom_id, None);
        assert_eq!(me.device_id, None);
    }
}
