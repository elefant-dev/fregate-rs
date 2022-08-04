use fregate::axum::routing::get;
use fregate::{axum::Router, init_tracing, AlwaysReadyAndAlive, Application};

async fn handler() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    init_tracing();

    Application::new_with_health(AlwaysReadyAndAlive::default())
        .rest_router(Router::new().route("/", get(handler)))
        .run()
        .await
        .unwrap();
}
