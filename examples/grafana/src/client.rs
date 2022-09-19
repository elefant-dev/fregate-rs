mod proto {
    tonic::include_proto!("hello");
}

use fregate::{bootstrap, Empty};
use opentelemetry::global::shutdown_tracer_provider;
use opentelemetry::propagation::Injector;
use proto::{hello_client::HelloClient, HelloRequest, HelloResponse};
use tonic::transport::Channel;
use tonic::{
    metadata::{MetadataKey, MetadataValue},
    Request, Response, Status,
};
use tracing::{info, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

// This code is taken from:
// https://github.com/open-telemetry/opentelemetry-rust/blob/main/examples/grpc/src/client.rs
pub struct MetadataMap<'a>(pub &'a mut tonic::metadata::MetadataMap);

impl<'a> Injector for MetadataMap<'a> {
    /// Set a key and value in the MetadataMap.  Does nothing if the key or value are not valid inputs
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = MetadataKey::from_bytes(key.as_bytes()) {
            if let Ok(val) = MetadataValue::try_from(value) {
                self.0.insert(key, val);
            }
        }
    }
}

pub fn inject_context<T>(request: &mut Request<T>) {
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(
            &Span::current().context(),
            &mut MetadataMap(request.metadata_mut()),
        )
    });
}

#[tracing::instrument(name = "request")]
async fn send_hello(
    client: &mut HelloClient<Channel>,
    mut request: Request<HelloRequest>,
) -> Result<Response<HelloResponse>, Status> {
    inject_context(&mut request);

    info!("Outgoing Request: {:?}", request);
    let response = client.say_hello(request).await;
    info!("Incoming Response: {:?}", response);

    response
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("OTEL_SERVICE_NAME", "CLIENT");
    std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "http://0.0.0.0:4317");

    let _config = bootstrap::<Empty, _>([], None);

    let channel = tonic::transport::Endpoint::from_static("http://0.0.0.0:8000")
        .connect()
        .await
        .unwrap();

    let mut client = HelloClient::new(channel);
    let request = Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    send_hello(&mut client, request).await?;

    shutdown_tracer_provider();
    Ok(())
}
