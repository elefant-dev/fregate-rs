use fregate::hyper::client::HttpConnector;
use fregate::hyper::Uri;
use fregate::logging::init_tracing;
use fregate::{hyper, tokio, tonic, tower, tracing};
use resources::proto::hello::{hello_client::HelloClient, HelloRequest};
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tonic::Request;
use tracing::info;

// This example is taken from: https://github.com/hyperium/tonic/blob/master/examples/src/tls/client_rustls.rs

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const CERTIFICATE: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../examples_resources/certs/ca.pem"
    ));
    init_tracing("info", "info", "0.0.0", "fregate", "client", None).unwrap();

    let mut roots = RootCertStore::empty();

    let mut buf = std::io::BufReader::new(CERTIFICATE);
    let certs = rustls_pemfile::certs(&mut buf)?;
    roots.add_parsable_certificates(&certs);

    let tls = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();

    let mut http = HttpConnector::new();
    http.enforce_http(false);

    // We have to do some wrapping here to map the request type from
    // `https://example.com` -> `https://[::1]:50051` because `rustls`
    // doesn't accept ip's as `ServerName`.
    let connector = tower::ServiceBuilder::new()
        .layer_fn(move |s| {
            let tls = tls.clone();

            hyper_rustls::HttpsConnectorBuilder::new()
                .with_tls_config(tls)
                .https_or_http()
                .enable_http2()
                .wrap_connector(s)
        })
        // Since our cert is signed with `example.com` but we actually want to connect
        // to a local server we will override the Uri passed from the `HttpsConnector`
        // and map it to the correct `Uri` that will connect us directly to the local server.
        .map_request(|_| Uri::from_static("https://0.0.0.0:8000"))
        .service(http);

    let client = hyper::Client::builder().http2_only(true).build(connector);

    // Using `with_origin` will let the codegenerated client set the `scheme` and
    // `authority` from the porvided `Uri`.
    let uri = Uri::from_static("https://example.com");

    let mut client = HelloClient::with_origin(client, uri);
    let request = Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    info!("Sending Request: {request:?}");
    let response = client.say_hello(request).await?;
    info!("Received Response: {response:?}");

    Ok(())
}
