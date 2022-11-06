use std::borrow::Borrow;

use actix_web::HttpRequest;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::*;

use crate::{model::UserId, config::COOKIE_KEY};

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

pub fn generate_auth<T: Borrow<UserJwt>>(user: T) -> Option<String> {
    let user = user.borrow();
    let iat = Utc::now();
    let exp = iat + Duration::hours(crate::config::TOKEN_LIFETIME);
    let claims = Claims {
        exp: exp.timestamp() as usize,
        user: user.clone(),
    };

    match encode(&Header::default(), &claims, &EncodingKey::from_secret(crate::config::SALT.as_ref())) {
        Ok(token) => Some(token),
        Err(_) => None,
    }
}

pub fn validate_auth<T: Borrow<str>>(token: T) -> Option<Claims> {
    let token = token.borrow();
    let validation = Validation::default();
    match decode::<Claims>(token, &DecodingKey::from_secret(crate::config::SALT.as_ref()), &validation) {
        Ok(c) => Some(c.claims),
        Err(error) => {
            println!("jwt error: {:?}", error);
            None
        }
    }
}

pub fn parse_request(req: &HttpRequest) -> Option<Claims> {
    match req.cookies() {
        Ok(cookies) => match cookies.iter().find(|c| c.name() == COOKIE_KEY) {
            Some(cookie) => validate_auth(cookie.value()),
            None => match req.headers().get(COOKIE_KEY) {
                Some(cookie) => validate_auth(cookie.to_str().unwrap_or_default()),
                None => None,
            },
        },
        Err(_) => match req.headers().get(COOKIE_KEY) {
            Some(cookie) => validate_auth(cookie.to_str().unwrap_or_default()),
            None => None,
        },
    }
}
