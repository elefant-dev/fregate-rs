use axum::{middleware::from_fn, routing::get, Router};
use fregate::{
    axum, bootstrap,
    extensions::RouterTonicExt,
    middleware::{trace_request, Attributes},
    tokio, Application, Empty,
};
use resources::{
    deny_middleware,
    grpc::{MyEcho, MyHello},
    proto::{echo::echo_server::EchoServer, hello::hello_server::HelloServer},
};

#[tokio::main]
async fn main() {
    const TLS_KEY_FULL_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../examples_resources/certs/tls.key"
    );
    const TLS_CERTIFICATE_FULL_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../examples_resources/certs/tls.cert"
    );

    std::env::set_var("OTEL_SERVER_TLS_KEY_PATH", TLS_KEY_FULL_PATH);
    std::env::set_var("OTEL_SERVER_TLS_CERT_PATH", TLS_CERTIFICATE_FULL_PATH);

    let config = bootstrap::<Empty, _>([]).unwrap();
    let attributes = Attributes::new_from_config(&config);

    let echo_service = EchoServer::new(MyEcho);
    let hello_service = HelloServer::new(MyHello);

    let rest = Router::new().route("/", get(|| async { "Hello, World!" }));

    // Echo service will always deny request
    let grpc = Router::from_tonic_service(echo_service)
        .layer(from_fn(deny_middleware))
        .merge(Router::from_tonic_service(hello_service));

    let app_router = rest.merge(grpc).layer(from_fn(move |req, next| {
        trace_request(req, next, attributes.clone())
    }));

    Application::new(&config)
        .router(app_router)
        .serve_tls()
        .await
        .unwrap();
}

/*
    grpcurl -insecure -import-path ./proto -proto hello.proto -d '{"name": "Tonic"}' 0.0.0.0:8000 hello.Hello/SayHello
    grpcurl -insecure -import-path ./proto -proto echo.proto -d '{"message": "Echo"}' 0.0.0.0:8000 echo.Echo/ping
    curl --insecure https://0.0.0.0:8000
    curl --insecure https://0.0.0.0:8000/health
    curl --insecure https://0.0.0.0:8000/ready
    curl --insecure https://0.0.0.0:8000/live
*/
