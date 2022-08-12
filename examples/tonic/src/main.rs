use fregate::axum::middleware::{from_fn, Next};
use fregate::axum::response::IntoResponse;
use fregate::axum::routing::get;
use fregate::axum::Router;
use fregate::hyper::Request;
use fregate::tonic::{Request as TonicRequest, Response as TonicResponse, Status};
use fregate::{
    grpc_trace_layer, http_trace_layer, init_tracing, AppConfig, Application, Tonicable,
};
use proto::{
    echo_server::{Echo, EchoServer},
    hello_server::{Hello, HelloServer},
    EchoRequest, EchoResponse, HelloRequest, HelloResponse,
};

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
        request: TonicRequest<HelloRequest>,
    ) -> Result<TonicResponse<HelloResponse>, Status> {
        let reply = HelloResponse {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(TonicResponse::new(reply))
    }
}

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

async fn deny_middleware<B>(_req: Request<B>, _next: Next<B>) -> impl IntoResponse {
    Status::permission_denied("You shall not pass").to_http()
}

#[tokio::main]
async fn main() {
    init_tracing();

    let echo_service = EchoServer::new(MyEcho);
    let hello_service = HelloServer::new(MyHello);

    let rest = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(http_trace_layer());

    // Echo service will always deny request
    let grpc = Router::from_tonic_service(echo_service)
        .layer(from_fn(deny_middleware))
        .merge(Router::from_tonic_service(hello_service))
        .layer(grpc_trace_layer());

    Application::new(AppConfig::default())
        .rest_router(rest)
        .grpc_router(grpc)
        .serve()
        .await
        .unwrap();
}

/*
    grpcurl -plaintext -import-path ./proto -proto hello.proto -d '{"name": "Tonic"}' 0.0.0.0:8000 hello.Hello/SayHello
    grpcurl -plaintext -import-path ./proto -proto echo.proto -d '{"message": "Echo"}' 0.0.0.0:8000 echo.Echo/ping
    curl http://0.0.0.0:8000
    curl http://0.0.0.0:8000/health
    curl http://0.0.0.0:8000/ready
    curl http://0.0.0.0:8000/live
*/
