use std::{sync::Mutex, collections::VecDeque};

use general::{auth::UserJwt, client::ClientTrait};

#[derive(Debug)]
pub struct DummyClient {
    pub messages: Mutex<VecDeque<String>>,
    pub auth: Mutex<UserJwt>
}

impl ClientTrait for DummyClient {
    fn send(&self, message: String) {
        self.messages.lock().unwrap().push_back(message)
    }

    fn authenticated(&self, auth: UserJwt) {
        let mut self_auth = self.auth.lock().unwrap();
        self_auth.email = auth.email;
        self_auth.id = auth.id;
        self_auth.name = auth.name;
        self_auth.session = auth.session;
    }
}

impl Default for DummyClient {
    fn default() -> Self {
        Self {
            messages: Mutex::default(),
            auth: Mutex::new(UserJwt::default())
        }
    }
}
