use fregate::extensions::{ReqwestExt, TonicReqExt};
use fregate::hyper::StatusCode;
use fregate::observability::init_tracing;
use fregate::{tokio, tonic, tracing};
use opentelemetry::global::shutdown_tracer_provider;
use reqwest::Url;
use resources::proto::hello::{hello_client::HelloClient, HelloRequest, HelloResponse};
use tonic::transport::Channel;
use tonic::{Request, Response, Status};
use tracing::info;

#[tracing::instrument(name = "get_check_status")]
async fn get_check_status() -> StatusCode {
    let http_client = reqwest::Client::new();

    let response = http_client
        .get(Url::parse("http://0.0.0.0:8000/check").unwrap())
        .inject_from_current_span()
        .send()
        .await
        .unwrap();

    response.status()
}

#[tracing::instrument(name = "send_hello")]
async fn send_hello(
    client: &mut HelloClient<Channel>,
    mut request: Request<HelloRequest>,
) -> Result<Response<HelloResponse>, Status> {
    if get_check_status().await == StatusCode::OK {
        request.inject_from_current_span();

        info!("Outgoing Request: {:?}", request);
        let response = client.say_hello(request).await;
        info!("Incoming Response: {:?}", response);
        response
    } else {
        Err(Status::cancelled("Service is unhealthy"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = init_tracing(
        "info",
        "info",
        "0.0.0",
        "fregate",
        "client",
        Some("http://0.0.0.0:4317"),
        None,
        None,
        None,
    )
    .unwrap();

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
