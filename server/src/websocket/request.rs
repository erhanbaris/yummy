use general::model::SessionToken;

use manager::api::user::UpdateUserField;
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Deserialize, Clone, Debug, Default)]
pub enum CommunicationFormat {
    #[default]
    #[serde(rename = "json")]
    Json,

    #[serde(rename = "flat")]
    FlatBuffer,

    #[serde(rename = "proto")]
    ProtoBuffer,
}


#[derive(Debug, Deserialize, Default)]
pub struct ConnectionInfo {
    #[serde(default)]
    #[serde(rename = "user")]
    pub user_id: Option<Uuid>,

    #[serde(default)]
    pub session_token: Option<SessionToken>,

    #[serde(default)]
    #[serde(rename = "format")]
    pub communication_format: Option<CommunicationFormat>,
}

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
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "user_type")]
pub enum UserType {
    Me,
    Get {
        user: Uuid
    },
    Update {
        name: Option<Option<String>>,
        email: Option<String>,
        password: Option<String>,
        device_id: Option<Option<String>>,
        custom_id: Option<Option<String>>,
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