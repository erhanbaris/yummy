use std::fmt::Debug;

use crate::auth::UserJwt;

pub trait ClientTrait: Debug {
    fn send(&self, message: String);
    fn authenticated(&self, user: UserJwt);
}

#[derive(Default, Debug)]
pub struct EmptyClient;

impl ClientTrait for EmptyClient {
    fn send(&self, _: String) {
        println!("EmptyClient received message");
    }

    fn authenticated(&self, _: UserJwt) {
        println!("EmptyClient authenticated");
    }
}