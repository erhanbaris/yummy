use std::{sync::Mutex, collections::VecDeque};

use crate::{client::ClientTrait, auth::UserJwt};

#[derive(Debug)]
pub struct DummyClient {
    pub messages: Mutex<VecDeque<String>>,
    pub auth: Mutex<UserJwt>
}

impl ClientTrait for DummyClient {
    fn send(&self, message: String) {

        println!("send {} {:?}", message, self.auth.lock().unwrap().id.get());
        self.messages.lock().unwrap().push_back(message)
    }

    fn authenticated(&self, auth: UserJwt) {
        println!(">> Auth id: {}", auth.id.get());
        let mut self_auth = self.auth.lock().unwrap();
        self_auth.email = auth.email;
        self_auth.id = auth.id;
        self_auth.name = auth.name;
        self_auth.session = auth.session;
    }
}

impl Default for DummyClient {
    fn default() -> Self {
        println!("DummyClient");
        Self {
            messages: Mutex::default(),
            auth: Mutex::new(UserJwt::default())
        }
    }
}

#[cfg(feature = "stateless")]
pub fn cleanup_redis(_redis: r2d2::Pool<redis::Client>) {
   /*match redis.get() {
        Ok(mut redis) => redis::cmd("flushall").execute(&mut redis),
        Err(_) => ()
    };*/
}