use fregate::axum::routing::get;
use fregate::{axum::Router, bootstrap, Application, Empty};

async fn handler() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    let config = bootstrap::<Empty, _>([]);

    Application::new(&config)
        .router(Router::new().route("/", get(handler)))
        .serve()
        .await
        .unwrap();
}
