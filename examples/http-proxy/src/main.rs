use fregate::axum::{
    routing::{get, post},
    Router,
};
use fregate::hyper::{Client, StatusCode};
use fregate::{http_trace_layer, init_tracing, AppConfig, Application, ProxyLayer};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    init_tracing();

    std::env::set_var("APP_PROXY_HOST", "127.0.0.1");
    std::env::set_var("APP_PROXY_PORT", "3000");

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

    let app = Router::new()
        .nest("/app", hello.merge(world).merge(might_be_proxied))
        .layer(http_trace_layer());

    Application::new(&AppConfig::default())
        .router(app)
        .serve()
        .await
        .unwrap();
}

async fn server() {
    let config = AppConfig::builder()
        .add_env_prefixed("APP_PROXY")
        .build()
        .unwrap();

    let app = Router::new()
        .route("/proxy_server/*path", get(|| async { "Hello, Proxy!" }))
        .route(
            "/proxy_server/*path",
            post(|| async { (StatusCode::BAD_REQUEST, "Probably You Want GET Method") }),
        );

    Application::new(&config).router(app).serve().await.unwrap();
}

/*
 -- 50% of requests handled localy other 50% proxied
    curl http://0.0.0.0:8000/app/proxy_server/abcd
 -- regular routes:
    curl http://0.0.0.0:8000/app/hello
    curl http://0.0.0.0:8000/app/world
*/
