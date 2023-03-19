use actix_codec::Framed;
use actix_tls::connect::rustls::webpki_roots_cert_store;
use awc::ws::Codec;
use awc::BoxedSocket;
use awc::ws::Frame;
use futures::SinkExt;
use futures::StreamExt;
use rustls::Certificate;
use rustls::ServerName;
use rustls::client::ServerCertVerified;
use rustls::client::ServerCertVerifier;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::SystemTime;
use rustls::ClientConfig;

pub struct NoCertificateVerification;

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}

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
        let mut config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(webpki_roots_cert_store())
            .with_no_client_auth();

        config
            .dangerous()
            .set_certificate_verifier(Arc::new(NoCertificateVerification));

        let url_with_query_param = format!("{0}?{1}={2}", url, query_param_name, key);
        let client = awc::Client::builder()
            .connector(awc::Connector::new().rustls(Arc::new(config)))
            .finish()
            .ws(url_with_query_param);

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
            _ => None
        }
    }

    pub async fn get_ping(&mut self) -> bool {
        let message = self.socket.next().await;
        matches!(message, Some(Ok(Frame::Ping(_))))
    }

    pub async fn get_pong(&mut self) -> bool {
        let message = self.socket.next().await;
        matches!(message, Some(Ok(Frame::Pong(_))))
    }

    pub async fn ping(&mut self) {
        self.socket.send(awc::ws::Message::Ping("".into())).await.unwrap();
    }

    pub async fn pong(&mut self) {
        self.socket.send(awc::ws::Message::Pong("".into())).await.unwrap();
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
