use std::{borrow::Borrow, sync::Arc};

use actix_web::HttpRequest;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::*;

use crate::{model::{UserId, SessionId}, config::YummyConfig};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserJwt {
    pub id: UserId,
    pub session: SessionId,
    pub name: Option<String>,
    pub email: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub user: UserJwt,
}

pub fn generate_auth<T: Borrow<UserJwt>>(config: Arc<YummyConfig>, user: T) -> Option<String> {
    let user = user.borrow();
    let iat = Utc::now();
    let exp = iat + Duration::seconds(config.token_lifetime);
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
        Ok(c) => match c.claims.exp > Utc::now().timestamp() as usize {
            true =>  Some(c.claims),
            false => None
        },
        Err(_) => {
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
