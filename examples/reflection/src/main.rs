#![allow(clippy::derive_partial_eq_without_eq)]

use axum::{middleware::from_fn, routing::get, Router};
use fregate::{
    axum, bootstrap,
    extensions::RouterTonicExt,
    middleware::{trace_request, Attributes},
    tokio, Application, Empty,
};
use resources::{grpc::MyEcho, proto::echo::echo_server::EchoServer, FILE_DESCRIPTOR_SET};

#[tokio::main]
async fn main() {
    let config = bootstrap::<Empty, _>([]).unwrap();
    let attributes = Attributes::new_from_config(&config);

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
            trace_request(req, next, attributes.clone())
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
