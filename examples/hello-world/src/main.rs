use fregate::axum::routing::get;
use fregate::{axum::Router, init_logging, AppConfig, Application};

async fn handler() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    init_logging();

    let config = AppConfig::default();

    Application::new_without_health(config)
        .rest_router(Router::new().route("/", get(handler)))
        .serve()
        .await
        .unwrap();
}
