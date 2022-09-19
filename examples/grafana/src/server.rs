use fregate::axum::Router;
use fregate::tonic::{Request as TonicRequest, Response as TonicResponse, Status};
use fregate::{bootstrap, grpc_trace_layer, Application, Empty, Tonicable};
use proto::{
    hello_server::{Hello, HelloServer},
    HelloRequest, HelloResponse,
};

mod proto {
    tonic::include_proto!("hello");
}

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
    std::env::set_var("OTEL_SERVICE_NAME", "SERVER");
    std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "http://0.0.0.0:4317");
    std::env::set_var("OTEL_PORT", "3000");

    let config = bootstrap::<Empty, _>([], None);

    let hello_service = HelloServer::new(MyHello);
    let grpc = Router::from_tonic_service(hello_service).layer(grpc_trace_layer());

    Application::new(&config)
        .router(grpc)
        .serve()
        .await
        .unwrap();
}
