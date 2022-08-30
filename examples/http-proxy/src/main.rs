use fregate::axum::routing::any;
use fregate::axum::{routing::get, Router};
use fregate::hyper::Client;
use fregate::{http_trace_layer, init_tracing, AppConfig, Application, ProxyLayer};

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

    let might_be_proxied = Router::new()
        .route("/proxy_server/*path", get(|| async { "Not Proxied" }))
        .layer(ProxyLayer::new(
            |_request| false,
            client,
            "http://127.0.0.1:3000",
        ));

    let app = Router::new()
        .nest("/app", hello.merge(world).merge(might_be_proxied))
        .layer(http_trace_layer());

    Application::new(&AppConfig::default())
        .rest_router(app)
        .serve()
        .await
        .unwrap();
}

async fn server() {
    let config = AppConfig::builder()
        .add_env_prefixed("APP_PROXY")
        .build()
        .unwrap();

    let app = Router::new().route("/proxy_server/*path", any(|| async { "Hello, Proxy!" }));

    Application::new(&config)
        .rest_router(app)
        .serve()
        .await
        .unwrap();
}
