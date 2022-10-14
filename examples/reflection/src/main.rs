#![allow(clippy::derive_partial_eq_without_eq)]

use fregate::axum::middleware::from_fn;
use fregate::axum::{routing::get, Router};
use fregate::tokio;
use fregate::tonic::{self, Request as TonicRequest, Response as TonicResponse, Status};
use fregate::{
    bootstrap, extensions::RouterTonicExt, middleware::trace_request, Application, Empty,
};
use resources::{
    proto::{
        echo::echo_server::{Echo, EchoServer},
        echo::{EchoRequest, EchoResponse},
    },
    FILE_DESCRIPTOR_SET,
};

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
    let config = bootstrap::<Empty, _>([]).unwrap();
    let conf = config.clone();

    let echo_service = EchoServer::new(MyEcho);

    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let reflection = Router::from_tonic_service(service);

    let rest = Router::new().route("/", get(|| async { "Hello, World!" }));
    let grpc = Router::from_tonic_service(echo_service);

    let app_router = rest
        .merge(grpc)
        .merge(reflection)
        .layer(from_fn(move |req, next| {
            trace_request(req, next, conf.clone())
        }));

    Application::new(&config)
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
