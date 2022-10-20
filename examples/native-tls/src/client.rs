use fregate::hyper::client::HttpConnector;
use fregate::hyper::{Client, Uri};
use fregate::logging::init_tracing;
use fregate::{tokio, tonic, tracing};
use hyper_tls::{
    native_tls::{Certificate, TlsConnector},
    HttpsConnector,
};
use resources::proto::hello::{hello_client::HelloClient, HelloRequest};
use tonic::Request;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const CERTIFICATE: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../examples_resources/certs/tls.cert"
    ));

    init_tracing("info", "info", "0.0.0", "fregate", "client", None).unwrap();

    let mut http = HttpConnector::new();
    http.enforce_http(false);

    let mut tls_connector_builder = TlsConnector::builder();
    let certificate = Certificate::from_pem(CERTIFICATE).unwrap();
    tls_connector_builder.add_root_certificate(certificate);
    tls_connector_builder.danger_accept_invalid_certs(true);
    tls_connector_builder.disable_built_in_roots(true);
    let tls = tls_connector_builder.build().unwrap().into();

    let connector = HttpsConnector::from((http, tls));
    let client = Client::builder().http2_only(true).build(connector);

    let uri = Uri::from_static("https://0.0.0.0:8000");

    let mut client = HelloClient::with_origin(client, uri);
    let request = Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    info!("Sending Request: {request:?}");
    let response = client.say_hello(request).await?;
    info!("Received Response: {response:?}");

    Ok(())
}
