mod proto {
    tonic::include_proto!("hello");
}

use proto::{
    hello_client::HelloClient,
    HelloRequest,
};

use tonic::{Request};
use tracing::{ info, info_span, Instrument };
use opentelemetry::propagation::Injector;
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub struct MetadataMap<'a>(pub &'a mut tonic::metadata::MetadataMap);

impl<'a> Injector for MetadataMap<'a> {
    /// Set a key and value in the MetadataMap.  Does nothing if the key or value are not valid inputs
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = tonic::metadata::MetadataKey::from_bytes(key.as_bytes()) {
            if let Ok(val) = tonic::metadata::MetadataValue::from_str(&value) {
                self.0.insert(key, val);
            }
        }
    }
}

pub fn inject<T>(mut request: tonic::Request<T>) -> tonic::Request<T> {
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(
            &tracing::Span::current().context(),
            &mut MetadataMap(request.metadata_mut()),
        )
    });
    request
}

#[tracing::instrument]
async fn send_hello() -> Result<(), Box<dyn std::error::Error>> {
    let channel = tonic::transport::Endpoint::from_static("http://0.0.0.0:8000")
        .connect()
        .await
        .unwrap();
    let mut client = HelloClient::new(channel);        

    let request = Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(inject(request)).instrument(info_span!("GreeterClient client request")).await?;

    info!("RESPONSE={:?}", response);
    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    std::env::set_var("OTEL_SERVICE_NAME", "CLINT SIDE");
    std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "http://0.0.0.0:4317");
    let _config = &fregate::AppConfig::default();
    
    send_hello().await?;
    
    Ok(())
}