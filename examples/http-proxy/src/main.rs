use fregate::axum::middleware::from_fn;
use fregate::axum::{
    routing::{get, post},
    Router, Server,
};
use fregate::hyper::{Client, StatusCode};
use fregate::tokio;
use fregate::{
    bootstrap,
    middleware::{trace_request, ProxyLayer},
    Application, Empty,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let config = bootstrap::<Empty, _>([]).unwrap();

    // Start server where to proxy requests
    tokio::spawn(server());

    // Create HTTP client
    let client = Client::new();

    // set up your server Routers
    let hello = Router::new().route("/hello", get(|| async { "Hello" }));
    let world = Router::new().route("/world", get(|| async { "World" }));

    let counter = Arc::new(AtomicU64::new(0));

    let might_be_proxied = Router::new()
        .route("/proxy_server/*path", get(|| async { "Not Proxied" }))
        .layer(ProxyLayer::new(
            move |_request| {
                let current = counter.fetch_add(1, Ordering::SeqCst);
                current % 2 == 0
            },
            client,
            "http://127.0.0.1:3000",
        ));

    let conf = config.clone();
    let app = Router::new()
        .nest("/app", hello.merge(world).merge(might_be_proxied))
        .layer(from_fn(move |req, next| {
            trace_request(req, next, conf.clone())
        }));

    Application::new(&config).router(app).serve().await.unwrap();
}

async fn server() {
    let app = Router::new()
        .route("/proxy_server/*path", get(|| async { "Hello, Proxy!" }))
        .route(
            "/proxy_server/*path",
            post(|| async { (StatusCode::BAD_REQUEST, "Probably You Want GET Method") }),
        );

    Server::bind(&SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        3000,
    ))
    .serve(app.into_make_service())
    .await
    .unwrap();
}

/*
 -- 50% of requests handled localy other 50% proxied
    curl http://0.0.0.0:8000/app/proxy_server/abcd
 -- regular routes:
    curl http://0.0.0.0:8000/app/hello
    curl http://0.0.0.0:8000/app/world
*/
