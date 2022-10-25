use fregate::{hyper, tonic};
use hyper::{client::HttpConnector, Client, Uri};
use hyper_tls::{native_tls, HttpsConnector};
use resources::proto::{
    echo::{echo_client::EchoClient, EchoRequest},
    hello::{hello_client::HelloClient, HelloRequest},
};
use tonic::Request;

#[tokio::main]
async fn main() {
    let mut http = HttpConnector::new();
    http.enforce_http(false);

    let tls_connector = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
        .into();

    let https = HttpsConnector::from((http, tls_connector));
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
