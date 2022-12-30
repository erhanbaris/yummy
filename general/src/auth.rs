use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::{borrow::Borrow, sync::Arc};
use std::future::Future;
use std::pin::Pin;

use actix_web::error::ErrorUnauthorized;
use actix_web::web::{Data, self};
use actix_web::{HttpRequest, FromRequest};
use actix_web::Error;
use actix_web::dev::Payload;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::*;

use crate::{model::{UserId, SessionId}, config::YummyConfig};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserJwt {
    pub id: Arc<UserId>,
    pub session: SessionId,
    pub name: Option<String>,
    pub email: Option<String>
}

#[derive(Debug, Serialize)]
pub struct ClaimsSerialize<'a> {
    pub exp: usize,
    pub user: &'a UserJwt,
}


#[derive(Debug, Deserialize)]
pub struct ClaimsDeserialize {
    pub exp: usize,
    pub user: UserJwt,
}

pub fn generate_auth<T: Borrow<UserJwt>>(config: Arc<YummyConfig>, token: T) -> Option<String> {
    let token = token.borrow();
    let iat = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0));
    let exp = iat + config.token_lifetime;
    let claims = ClaimsSerialize {
        exp: exp.as_secs() as usize,
        user: token,
    };

    match encode(&Header::default(), &claims, &EncodingKey::from_secret(config.salt_key.as_ref())) {
        Ok(token) => Some(token),
        Err(_) => None,
    }
}

pub fn validate_auth<T: Borrow<str>>(config: Arc<YummyConfig>, token: T) -> Option<ClaimsDeserialize> {
    let token = token.borrow();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|item| item.as_secs() as usize).unwrap_or_default();
    let validation = Validation::default();
    match decode::<ClaimsDeserialize>(token, &DecodingKey::from_secret(config.salt_key.as_ref()), &validation) {
        Ok(c) => match c.claims.exp > now {
            true =>  Some(c.claims),
            false => None
        },
        Err(_) => {
            None
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiIntegration;

impl FromRequest for ApiIntegration {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<ApiIntegration, Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {

        let config = match req.app_data::<Data<Arc<YummyConfig>>>() {
            Some(config) => config,
            None => return Box::pin(async { Err(ErrorUnauthorized("unauthorized")) })
        };

        let api_key = match req.headers().get(&config.api_key_name) {
            Some(value) =>  match value.to_str() {
                Ok(value) => value.to_string(),
                Err(_) => return Box::pin(async { Err(ErrorUnauthorized("unauthorized")) })
            }
            None => match web::Query::<HashMap<String, String>>::from_query(req.query_string()) {
                Ok(map) => match map.0.get(&config.api_key_name) {
                    Some(value) => value.to_string(),
                    None => return Box::pin(async { Err(ErrorUnauthorized("unauthorized")) })
                },
                Err(_) => return Box::pin(async { Err(ErrorUnauthorized("unauthorized")) })
            }
        };

        if api_key != config.integration_key {
            return Box::pin(async { Err(ErrorUnauthorized("unauthorized")) })
        }

        Box::pin(async move { Ok(ApiIntegration) })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserAuth {
    pub user: UserId,
    pub session: SessionId
}

impl UserAuth {
    pub fn empty() -> Self {
        Self {
            user: UserId::from(uuid::Uuid::nil()),
            session: SessionId::from(uuid::Uuid::nil())
        }
    }

    pub fn is_empty(&self) -> bool {
        self.user.is_empty()
    }
}
