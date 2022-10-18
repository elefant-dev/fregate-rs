use axum::{
    middleware::{from_fn, Next},
    response::IntoResponse,
    routing::get,
    Router,
};
use fregate::{
    axum, bootstrap,
    extensions::RouterTonicExt,
    hyper,
    middleware::{trace_request, Attributes},
    tokio, tonic, Application, Empty,
};
use hyper::Request;
use resources::{
    proto::{
        echo::{
            echo_server::{Echo, EchoServer},
            EchoRequest, EchoResponse,
        },
        hello::{
            hello_server::{Hello, HelloServer},
            HelloRequest, HelloResponse,
        },
    },
    TLS_CERT, TLS_KEY,
};
use tonic::{Request as TonicRequest, Response as TonicResponse, Status};

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
            message: format!("Hello From Tonic Server {}!", request.into_inner().name),
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
    std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "http://0.0.0.0:4317");

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
        .serve_tls(TLS_CERT, TLS_KEY)
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
