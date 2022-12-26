use std::fmt::Debug;

use actix::Recipient;

use crate::{auth::UserJwt,  state::SendMessage, model::UserId};

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

#[derive(Debug)]
pub struct StatelessClient(UserId, Recipient<SendMessage>);

impl ClientTrait for StatelessClient {
    fn send(&self, message: String) {
        println!("STATELESS MESSAGE SENT");
        self.1.do_send(SendMessage {
            user_id: self.0,
            message
        })
    }

    fn authenticated(&self, _: UserJwt) {
        println!("EmptyClient authenticated");
    }
}