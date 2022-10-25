use fregate::{hyper, tonic};
use hyper::{Client, Uri};
use hyper_rustls::{ConfigBuilderExt, HttpsConnectorBuilder};
use resources::proto::{
    echo::{echo_client::EchoClient, EchoRequest},
    hello::{hello_client::HelloClient, HelloRequest},
};
use rustls::client::{ServerCertVerified, ServerCertVerifier};
use rustls::{Certificate, ClientConfig, Error, ServerName};
use std::{sync::Arc, time::SystemTime};
use tonic::Request;

struct DummyServerCertVerifier;

impl ServerCertVerifier for DummyServerCertVerifier {
    fn verify_server_cert(
        &self,
        _: &Certificate,
        _: &[Certificate],
        _: &ServerName,
        _: &mut dyn Iterator<Item = &[u8]>,
        _: &[u8],
        _: SystemTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }
}

#[tokio::main]
async fn main() {
    let mut tls = ClientConfig::builder()
        .with_safe_defaults()
        .with_native_roots()
        .with_no_client_auth();
    tls.dangerous()
        .set_certificate_verifier(Arc::new(DummyServerCertVerifier));

    let https = HttpsConnectorBuilder::new()
        .with_tls_config(tls)
        .https_only()
        .enable_http1()
        .build();
    let hyper = Client::builder().http2_only(true).build(https);
    let origin: Uri = "https://localhost:8000".parse().unwrap();
    let mut echo_client = EchoClient::with_origin(hyper.clone(), origin.clone());
    let mut hello_client = HelloClient::with_origin(hyper, origin);

    let response = echo_client
        .ping(Request::new(EchoRequest {
            message: "Hello from Jindřich from Skalica!".to_string(),
        }))
        .await
        .unwrap_err();
    eprintln!("response: `{response:?}`.");

    let response = hello_client
        .say_hello(Request::new(HelloRequest {
            name: "Jindřich".to_owned(),
        }))
        .await
        .unwrap()
        .into_inner();
    println!("response: `{response:?}`.");
}
