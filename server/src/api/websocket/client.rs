use actix_codec::Framed;
use awc::ws::Codec;
use awc::BoxedSocket;
use awc::Client;
use awc::ws::Frame;
use futures::SinkExt;
use futures::StreamExt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::marker::PhantomData;
pub struct WebsocketTestClient<REQ, RES>
where
    REQ: Debug + Send + Serialize + DeserializeOwned,
    RES: Debug + Send + Serialize + DeserializeOwned,
{
    socket: Framed<BoxedSocket, Codec>,
    marker1: std::marker::PhantomData<REQ>,
    marker2: std::marker::PhantomData<RES>,
}

impl<REQ, RES> WebsocketTestClient<REQ, RES>
where
    REQ: Debug + Send + Serialize + DeserializeOwned,
    RES: Debug + Send + Serialize + DeserializeOwned,
{
    pub async fn new(url: String, query_param_name: String, key: String) -> Self {
        let url_with_query_param = format!("{0}?{1}={2}", url, query_param_name, key);
        let client = Client::default().ws(url_with_query_param);
        let (_, socket) = client.connect().await.unwrap();
        Self {
            socket,
            marker1: PhantomData,
            marker2: PhantomData,
        }
    }

    pub async fn disconnect(&mut self) {
        self.socket.close().await.unwrap();
    }

    pub async fn get_text(&mut self) -> Option<String> {
        let message = self.socket.next().await;
        match message {
            Some(Ok(Frame::Text(t))) => {
                let bytes = t.as_ref();
                let data = String::from_utf8(bytes.to_vec()).unwrap_or_default();
                Some(data)
            }
            Some(Ok(Frame::Ping(t))) => {
                self.socket.send(awc::ws::Message::Pong(t)).await.unwrap();
                Some(String::new())
            }
            Some(Ok(Frame::Pong(t))) => {
                self.socket.send(awc::ws::Message::Ping(t)).await.unwrap();
                Some(String::new())
            }
            _ => Some(String::new()),
        }
    }

    pub async fn send<R>(&mut self, message: R) where R: Clone + Serialize + DeserializeOwned {
        match serde_json::to_string(&message) {
            Ok(text) => self
                .socket
                .send(awc::ws::Message::Text(text.into()))
                .await
                .unwrap(),
            Err(error) => {
                panic!("------------ Serialize error : {:?}", error);
            }
        }
    }
}
