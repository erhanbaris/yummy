use std::collections::HashMap;

use cucumber::{given, then, when};
use general::websocket::WebsocketTestClient;

use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use super::{YummyWorld, ClientInfo};

fn get_user<'a>(world: &'a mut YummyWorld, user: &'a String) -> &'a mut ClientInfo {
    match world.ws_clients.get_mut(user) {
        Some(ws_client) => ws_client,
        None => panic!("User {} not connected", user),
    }
}

async fn get_message(world: &mut YummyWorld, user: &String) -> String {
    let client = get_user(world, &user);
    let mut retry_counter = 0;
    while retry_counter < 5 {
        let message = client.socket.get_text().await;
        if let Some(message) = message {
            return message;
        }
        retry_counter += 1;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    String::new()
}

async fn send_message(world: &mut YummyWorld, user: &String, message: Value) {
    let client = get_user(world, &user);
    client.socket.send(message).await;
}

async fn user_receive_message<'a, T: DeserializeOwned>(world: &'a mut YummyWorld, user: &'a String) -> (&'a mut ClientInfo, String, T) {
    let message = get_message(world, &user).await;
    assert!(!message.is_empty(), "No message received");

    let client = get_user(world, &user);
    let received_message = serde_json::from_str::<T>(&message).unwrap();
    (client, message, received_message)
}

/* Givens */
#[given(expr = "{word} connected")]
async fn user_connect(world: &mut YummyWorld, user: String) {
    let ws_client = WebsocketTestClient::<String, String>::new(world.ws_server.url("/v1/socket") , general::config::DEFAULT_API_KEY_NAME.to_string(), general::config::DEFAULT_DEFAULT_INTEGRATION_KEY.to_string()).await;
    world.ws_clients.insert(
        user,
        ClientInfo {
            socket: ws_client,
            last_message: None,
            room_id: None,
            name: None,
            last_error: None,
            message: String::new(),
            token: String::new(),
            memory: HashMap::default()
        },
    );
}

/* Whens */
#[when(expr = "{word} add {string} to memory as {string}")]
async fn add_to_memory(world: &mut YummyWorld, user: String, value: String, key: String) {
    let user = get_user(world, &user);
    user.memory.insert(key, value);
}

#[when(expr = "{word} try to refresh token")]
async fn refresh_token(world: &mut YummyWorld, user: String) {
    let token = {
        get_user(world, &user).token.clone()
    };

    send_message(world, &user, json!({
        "type": "RefreshToken",
        "token": token
    })).await;
}

#[when(expr = "{word} try to restore token")]
async fn restore_token(world: &mut YummyWorld, user: String) {
    let token = {
        get_user(world, &user).memory.get(&"token".to_string()).cloned().unwrap_or(String::new())
    };

    send_message(world, &user, json!({
        "type": "RestoreToken",
        "token": token
    })).await;
}

#[when(expr = "{word} set token to {string}")]
async fn set_token(world: &mut YummyWorld, user: String, token: String) {
    get_user(world, &user).token = token;
}

#[when(expr = "{word} save token to memory")]
async fn save_token(world: &mut YummyWorld, user: String) {
    let token = {
        get_user(world, &user).token.clone()
    };
    add_to_memory(world, user, token, "token".to_string()).await;
}

#[when(expr = "{word} send {string} as a json message")]
async fn send_json(world: &mut YummyWorld, user: String, message: String) {
    send_message(world, &user, serde_json::from_str(&message).unwrap_or_default()).await;
}

#[when(expr = "{word} authenticate via email with {string} {string}")]
async fn email_auth_with_parameters(world: &mut YummyWorld, user: String, email: String, password: String) {
    send_message(world, &user, json!({
        "type": "AuthEmail",
        "email": email,
        "password": password
    })).await;
}

#[when(expr = "{word} register via email with {string} {string}")]
async fn register_with_email(world: &mut YummyWorld, user: String, email: String, password: String) {
    send_message(world, &user, json!({
        "type": "AuthEmail",
        "email": email,
        "password": password,
        "create": true
    })).await;
}

#[when(expr = "{word} authenticate via custom id with {string}")]
async fn custom_id_auth_with_parameters(world: &mut YummyWorld, user: String, custom_id: String) {
    send_message(world, &user, json!({
        "type": "AuthCustomId",
        "id": custom_id
    })).await;
}


#[when(expr = "{word} register via custom id with {string}")]
async fn register_with_custom_id(world: &mut YummyWorld, user: String, custom_id: String) {
    send_message(world, &user, json!({
        "type": "AuthCustomId",
        "id": custom_id,
        "create": true
    })).await;
}


#[when(expr = "{word} authenticate via device id with {string}")]
async fn register_with_device_id(world: &mut YummyWorld, user: String, device_id: String) {
    send_message(world, &user, json!({
        "type": "AuthDeviceId",
        "id": device_id
    })).await;
}

#[when(expr = "{word} logout")]
async fn logout(world: &mut YummyWorld, user: String) {
    send_message(world, &user, json!({
        "type": "Logout"
    })).await;
}

/* Thens */
#[then(expr = "{word} authenticated")]
async fn authenticated(world: &mut YummyWorld, user: String) {
    let (client, message, received_message) = user_receive_message::<serde_json::Value>(world, &user).await;
    assert_eq!(received_message.as_object().unwrap().get("type").unwrap().as_str().unwrap(), "Authenticated");
    
    client.last_message = Some(message);
    client.token = received_message.as_object().unwrap().get("token").unwrap().as_str().unwrap().to_string();
}

#[then(expr = "{word} receive {word} message")]
async fn receive_message_type(world: &mut YummyWorld, user: String, message_type: String) {
    let (client, message, received_message) = user_receive_message::<serde_json::Value>(world, &user).await;
    assert_eq!(received_message.as_object().unwrap().get("type").unwrap().as_str().unwrap(), &message_type);

    client.last_message = Some(message);
}

#[then(expr = "{word} request succeeded")]
async fn succeeded(world: &mut YummyWorld, user: String) {
    let (client, message, received_message) = user_receive_message::<serde_json::Value>(world, &user).await;
    assert_eq!(received_message.as_object().unwrap().get("status").unwrap().as_bool().unwrap(), true);

    client.last_message = Some(message);
}

#[then(expr = "{word} request failed")]
async fn failed(world: &mut YummyWorld, user: String) {
    let (client, message, received_message) = user_receive_message::<serde_json::Value>(world, &user).await;
    assert_eq!(received_message.as_object().unwrap().get("status").unwrap().as_bool().unwrap(), false);

    client.last_error = Some(message);
}