use axum::{middleware::from_fn, Router};
use fregate::{
    axum, bootstrap,
    extensions::RouterTonicExt,
    middleware::{trace_request, Attributes},
    tokio, Application, ConfigSource, Empty,
};
use resources::{grpc::MyHello, proto::hello::hello_server::HelloServer};

#[tokio::main]
async fn main() {
    const TLS_KEY_FULL_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../examples_resources/certs/server.key"
    );
    const TLS_CERTIFICATE_FULL_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../examples_resources/certs/server.pem"
    );

    std::env::set_var("TEST_SERVER_TLS_KEY_PATH", TLS_KEY_FULL_PATH);
    std::env::set_var("TEST_SERVER_TLS_CERT_PATH", TLS_CERTIFICATE_FULL_PATH);

    let config = bootstrap::<Empty, _>([ConfigSource::EnvPrefix("TEST")]).unwrap();
    let attributes = Attributes::new_from_config(&config);

    let hello_service = HelloServer::new(MyHello);
    let grpc = Router::from_tonic_service(hello_service).layer(from_fn(move |req, next| {
        trace_request(req, next, attributes.clone())
    }));

    Application::new(&config)
        .router(grpc)
        .serve_tls()
        .await
        .unwrap();
}

/*
    grpcurl -insecure -import-path ./examples/examples_resources/proto -proto hello.proto -d '{"name": "Tonic"}' 0.0.0.0:8000 hello.Hello/SayHello
    grpcurl -insecure -import-path ./examples/examples_resources/proto -proto echo.proto -d '{"message": "Echo"}' 0.0.0.0:8000 echo.Echo/ping
    curl --insecure https://0.0.0.0:8000
    curl --insecure https://0.0.0.0:8000/health
    curl --insecure https://0.0.0.0:8000/ready
    curl --insecure https://0.0.0.0:8000/live
*/
