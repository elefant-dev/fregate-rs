use axum::routing::get;
use axum::Router;
use fregate::{AlwaysHealthy, Application};
use proto::{
    echo_server::{Echo, EchoServer},
    hello_server::{Hello, HelloServer},
    EchoRequest, EchoResponse, HelloRequest, HelloResponse,
};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

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
    let echo_service = EchoServer::new(MyEcho);
    let hello_service = HelloServer::new(MyHello);

    let grpc_router = Server::builder()
        .add_service(echo_service)
        .add_service(hello_service);

    let app = Application::builder::<AlwaysHealthy>()
        .init_tracing()
        .set_configuration_file("./src/resources/default_conf.toml")
        .set_rest_routes(Router::new().route("/", get(handler)))
        .set_grpc_routes(grpc_router)
        .build();

    app.run().await.unwrap();
}

/*
    grpcurl -plaintext -import-path ./proto -proto hello.proto -d '{"name": "Tonic"}' 0.0.0.0:5000 hello.Hello/SayHello
    grpcurl -plaintext -import-path ./proto -proto echo.proto -d '{"message": "Echo"}' 0.0.0.0:5000 echo.Echo/ping
    curl http://0.0.0.0:5000/v1
*/
