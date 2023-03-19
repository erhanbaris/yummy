use std::sync::Arc;

use actix::{Message, Recipient, Actor, Handler};
use futures_util::StreamExt;
use actix::AsyncContext;
use actix::Context;
use actix::WrapFuture;
use redis::{FromRedisValue, RedisError};
use yummy_model::config::YummyConfig;

pub trait PubSubMessage: Message + Send + 'static {
    fn new(message: String) -> Self;
}

pub async fn get_async_connection(config: Arc<YummyConfig>) -> anyhow::Result<redis::aio::Connection> {
    let client = redis::Client::open(config.redis_url.clone())?;
    Ok(client.get_async_connection().await?)
}

pub async fn subscribe_to_channel<M: PubSubMessage>(config: Arc<YummyConfig>, channel: String, receiver: Recipient<M>) where <M as actix::Message>::Result: std::marker::Send {
    let connection = get_async_connection(config).await.unwrap();
    let mut pubsub = connection.into_pubsub();
    pubsub.subscribe(channel).await.unwrap();

    tokio::spawn(async move {
        while let Some(msg) = pubsub.on_message().next().await {
            if let Ok(payload) = msg.get_payload().and_then(|payload| -> Result<String, RedisError> { FromRedisValue::from_redis_value(&payload) }) {
                receiver.do_send(M::new(payload));
            }
        }
    });
}

pub fn subscribe<M: PubSubMessage, T: Actor<Context = Context<T>> + Handler<M>>(me: &mut T, ctx: &mut T::Context, config: Arc<YummyConfig>, channel: String) where <M as actix::Message>::Result: std::marker::Send {
    let future = subscribe_to_channel::<M>(config, channel, ctx.address().recipient());
    let actor_fut = future.into_actor(me);
    ctx.wait(actor_fut);
}

