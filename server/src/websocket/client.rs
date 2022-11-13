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
    url: String,
    marker1: std::marker::PhantomData<REQ>,
    marker2: std::marker::PhantomData<RES>,
}

impl<REQ, RES> Debug for WebsocketTestClient<REQ, RES>
where
    REQ: Debug + Send + Serialize + DeserializeOwned,
    RES: Debug + Send + Serialize + DeserializeOwned,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WebsocketTestClient")
    }
}

impl<REQ, RES> WebsocketTestClient<REQ, RES>
where
    REQ: Debug + Send + Serialize + DeserializeOwned,
    RES: Debug + Send + Serialize + DeserializeOwned,
{
    pub async fn new(url: String) -> Self {
        let client = Client::default().ws(url.clone());
        let (_, socket) = client.connect().await.unwrap();
        Self {
            socket,
            url,
            marker1: PhantomData,
            marker2: PhantomData,
        }
    }

    pub async fn disconnect(&mut self) {
        self.socket.close().await.unwrap();
    }

    pub async fn reconnect(&mut self) {
        let client = Client::default().ws(self.url.clone());
        let (_, socket) = client.connect().await.unwrap();
        self.socket = socket;
    }

    pub async fn get_text(&mut self) -> Option<String> {
        let message = self.socket.next().await;
        match message {
            Some(Ok(Frame::Text(t))) => {
                let bytes = t.as_ref();
                let data = String::from_utf8(bytes.to_vec()).unwrap_or_default();
                Some(data)
            }
            Some(Ok(Frame::Binary(t))) => {
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
            Some(_) => Some(String::new()),
            None => None,
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

    pub async fn pong(&mut self) {
        match serde_json::to_string("PING") {
            Ok(text) => self
                .socket
                .send(awc::ws::Message::Pong(text.into()))
                .await
                .unwrap(),
            Err(error) => {
                panic!("------------ Serialize error : {:?}", error);
            }
        }
    }
}
