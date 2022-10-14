use fregate::axum::middleware::from_fn;
use fregate::axum::Router;
use fregate::middleware::trace_request;
use fregate::tokio;
use fregate::tonic::{self, Request as TonicRequest, Response as TonicResponse, Status};
use fregate::{bootstrap, extensions::RouterTonicExt, Application, Empty};
use resources::proto::hello::{
    hello_server::{Hello, HelloServer},
    HelloRequest, HelloResponse,
};

#[derive(Default)]
struct MyHello;

#[tonic::async_trait]
impl Hello for MyHello {
    async fn say_hello(
        &self,
        request: TonicRequest<HelloRequest>,
    ) -> Result<TonicResponse<HelloResponse>, Status> {
        let reply = HelloResponse {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(TonicResponse::new(reply))
    }
}

#[tokio::main]
async fn main() {
    std::env::set_var("OTEL_COMPONENT_NAME", "server");
    std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "http://0.0.0.0:4317");

    let config = bootstrap::<Empty, _>([]).unwrap();
    let conf = config.clone();

    let hello_service = HelloServer::new(MyHello);
    let grpc = Router::from_tonic_service(hello_service).layer(from_fn(move |req, next| {
        trace_request(req, next, conf.clone())
    }));

    Application::new(&config)
        .router(grpc)
        .serve()
        .await
        .unwrap();
}
