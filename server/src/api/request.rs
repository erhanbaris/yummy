use std::collections::HashMap;

use general::{model::{UserId, SessionId}, meta::MetaType};
use serde::{Deserialize, Serialize, Deserializer};

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "auth_type")]
pub enum AuthType {
    Email {
        email: String,
        password: String,

        #[serde(default, rename = "create")]
        if_not_exist_create: bool
    },
    DeviceId {
        id: String
    },
    CustomId {
        id: String
    },
    Refresh {
        token: String
    },
    Restore {
        token: String
    },
    Logout,
    StartUserTimeout {
        session_id: SessionId
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "user_type")]
pub enum UserType {
    Me,
    Get {
        user: UserId
    },
    Update {
        name: Option<String>,
        email: Option<String>,
        password: Option<String>,
        device_id: Option<String>,
        custom_id: Option<String>,

        meta: Option<HashMap<String, MetaType>>
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum Request {
    Auth {
        #[serde(flatten)]
        auth_type: AuthType
    },
    User {
        #[serde(flatten)]
        user_type: UserType
    }
}