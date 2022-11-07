use std::{borrow::Borrow, sync::Arc};

use actix_web::HttpRequest;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::*;

use crate::{model::UserId, config::YummyConfig};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserJwt {
    //#[serde(with = "crate::tool::hashid_usize")]
    pub id: UserId,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub user: UserJwt,
}

pub fn generate_auth<T: Borrow<UserJwt>>(config: Arc<YummyConfig>, user: T) -> Option<String> {
    let user = user.borrow();
    let iat = Utc::now();
    let exp = iat + Duration::hours(config.token_lifetime);
    let claims = Claims {
        exp: exp.timestamp() as usize,
        user: user.clone(),
    };

    match encode(&Header::default(), &claims, &EncodingKey::from_secret(config.salt_key.as_ref())) {
        Ok(token) => Some(token),
        Err(_) => None,
    }
}

pub fn validate_auth<T: Borrow<str>>(config: Arc<YummyConfig>, token: T) -> Option<Claims> {
    let token = token.borrow();
    let validation = Validation::default();
    match decode::<Claims>(token, &DecodingKey::from_secret(config.salt_key.as_ref()), &validation) {
        Ok(c) => Some(c.claims),
        Err(error) => {
            println!("jwt error: {:?}", error);
            None
        }
    }
}

pub fn parse_request(config: Arc<YummyConfig>, req: &HttpRequest) -> Option<Claims> {
    match req.cookies() {
        Ok(cookies) => match cookies.iter().find(|c| c.name() == config.salt_key) {
            Some(cookie) => validate_auth(config.clone(), cookie.value()),
            None => match req.headers().get(&config.cookie_key) {
                Some(cookie) => validate_auth(config.clone(), cookie.to_str().unwrap_or_default()),
                None => None,
            },
        },
        Err(_) => match req.headers().get(&config.cookie_key) {
            Some(cookie) => validate_auth(config.clone(), cookie.to_str().unwrap_or_default()),
            None => None,
        },
    }
}
