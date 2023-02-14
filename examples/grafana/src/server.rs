use axum::Router;
use fregate::axum::routing::get;
use fregate::hyper::StatusCode;
use fregate::{axum, bootstrap, extensions::RouterTonicExt, tokio, AppConfig, Application};
use resources::{grpc::MyHello, proto::hello::hello_server::HelloServer};

#[tokio::main]
async fn main() {
    std::env::set_var("OTEL_COMPONENT_NAME", "server");
    std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "http://0.0.0.0:4317");

    let config: AppConfig = bootstrap([]).unwrap();
    let hello_service = HelloServer::new(MyHello);

    let rest = Router::new().route("/check", get(|| async { StatusCode::OK }));
    let grpc = Router::from_tonic_service(hello_service);

    let router = grpc.merge(rest);

    Application::new(&config)
        .router(router)
        .serve()
        .await
        .unwrap();
}
