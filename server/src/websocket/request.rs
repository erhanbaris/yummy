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
    pub session_token: Option<String>,

    #[serde(default)]
    #[serde(rename = "format")]
    pub communication_format: Option<CommunicationFormat>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "auth_type", content = "content")]
pub enum AuthType {
    Email {
        email: String,
        password: String,
    },
    Custom(String),
    Device(String)
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum Request {
    Auth {
        auth_type: AuthType,

        #[serde(rename = "create")]
        if_not_exist_create: bool
    }
}