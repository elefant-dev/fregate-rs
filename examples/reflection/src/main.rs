#![allow(clippy::derive_partial_eq_without_eq)]

use fregate::axum::{routing::get, Router};
use fregate::tonic::{Request as TonicRequest, Response as TonicResponse, Status};
use fregate::{
    extensions::RouterTonicExt,
    init_tracing_from_config,
    middleware::{grpc_trace_layer, http_trace_layer},
    Application,
};
use fregate::{tokio, ApplicationConfig, TracingConfig};
use proto::{
    echo_server::{Echo, EchoServer},
    EchoRequest, EchoResponse,
};

mod proto {
    include!("echo.rs");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("echo_descriptor.bin");
}

#[derive(Default)]
struct MyEcho;

#[tonic::async_trait]
impl Echo for MyEcho {
    async fn ping(
        &self,
        request: TonicRequest<EchoRequest>,
    ) -> Result<TonicResponse<EchoResponse>, Status> {
        let reply = EchoResponse {
            message: request.into_inner().message,
        };

        Ok(TonicResponse::new(reply))
    }
}

#[tokio::main]
async fn main() {
    let conf = ApplicationConfig::default();
    let trace_conf = TracingConfig::default();

    init_tracing_from_config(trace_conf).unwrap();

    let echo_service = EchoServer::new(MyEcho);

    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let reflection = Router::from_tonic_service(service);

    let rest = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(http_trace_layer());

    let grpc = Router::from_tonic_service(echo_service).layer(grpc_trace_layer());
    let app_router = rest.merge(grpc).merge(reflection);

    Application::new(conf)
        .router(app_router)
        .serve()
        .await
        .unwrap();
}

/*
    grpcurl -plaintext 0.0.0.0:8000 list
    grpcurl -plaintext -d '{"message": "Echo"}' 0.0.0.0:8000 echo.Echo/ping
    curl http://0.0.0.0:8000
    curl http://0.0.0.0:8000/health
    curl http://0.0.0.0:8000/ready
    curl http://0.0.0.0:8000/live
*/
