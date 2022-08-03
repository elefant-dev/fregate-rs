use fregate::axum::routing::get;
use fregate::axum::Router;
use fregate::tonic::transport::Server;
use fregate::tonic::{Request, Response, Status};
use fregate::{init_tracing, AlwaysHealthy, Application};
use proto::{
    echo_server::{Echo, EchoServer},
    hello_server::{Hello, HelloServer},
    EchoRequest, EchoResponse, HelloRequest, HelloResponse,
};
use std::sync::Arc;

mod proto {
    tonic::include_proto!("hello");
    tonic::include_proto!("echo");
}

#[derive(Default)]
struct MyHello;

#[derive(Default)]
struct MyEcho;

#[tonic::async_trait]
impl Hello for MyHello {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let reply = HelloResponse {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }
}

#[tonic::async_trait]
impl Echo for MyEcho {
    async fn ping(&self, request: Request<EchoRequest>) -> Result<Response<EchoResponse>, Status> {
        let reply = EchoResponse {
            message: request.into_inner().message,
        };

        Ok(Response::new(reply))
    }
}

async fn handler() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    init_tracing();

    let echo_service = EchoServer::new(MyEcho);
    let hello_service = HelloServer::new(MyHello);

    let grpc_router = Server::builder()
        .add_service(echo_service)
        .add_service(hello_service);

    Application::new_with_health(Arc::new(AlwaysHealthy::default()))
        .rest_router(Router::new().route("/", get(handler)))
        .grpc_router(grpc_router)
        .run()
        .await
        .unwrap();
}

/*
    grpcurl -plaintext -import-path ./proto -proto hello.proto -d '{"name": "Tonic"}' 0.0.0.0:8000 hello.Hello/SayHello
    grpcurl -plaintext -import-path ./proto -proto echo.proto -d '{"message": "Echo"}' 0.0.0.0:8000 echo.Echo/ping
    curl http://0.0.0.0:8000/v1
    curl http://0.0.0.0:8000/health
*/
