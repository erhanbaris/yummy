use std::sync::Arc;

use actix::{Message, Recipient, Context};
use futures_util::StreamExt;
use redis::{FromRedisValue, RedisError};

use crate::config::YummyConfig;


#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct MessageReceived(String);

pub async fn get_async_connection(config: Arc<YummyConfig>) -> anyhow::Result<redis::aio::Connection> {
    let client = redis::Client::open(config.redis_url.clone())?;
    Ok(client.get_async_connection().await?)
}

pub async fn subscribe_to_channel(config: Arc<YummyConfig>, channel: String, receiver: Recipient<MessageReceived>) {
    let connection = get_async_connection(config).await.unwrap();
    let mut pubsub = connection.into_pubsub();
    pubsub.subscribe(channel).await.unwrap();

    tokio::spawn(async move {
        while let Some(msg) = pubsub.on_message().next().await {
            if let Ok(payload) = msg.get_payload().and_then(|payload| -> Result<String, RedisError> { FromRedisValue::from_redis_value(&payload) }) {
                receiver.do_send(MessageReceived(payload));
            }
        }
    });
}

