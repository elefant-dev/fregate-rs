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
