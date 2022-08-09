use fregate::axum::routing::get;
use fregate::{axum::Router, init_tracing, AppConfig, Application};

async fn handler() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    init_tracing();

    let config = AppConfig::default();

    Application::new(config)
        .rest_router(Router::new().route("/", get(handler)))
        .serve()
        .await
        .unwrap();
}
