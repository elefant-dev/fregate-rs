use fregate::axum::routing::get;
use fregate::{axum::Router, AppConfig, Application};

async fn handler() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    Application::new(&AppConfig::default())
        .router(Router::new().route("/", get(handler)))
        .serve()
        .await
        .unwrap();
}
