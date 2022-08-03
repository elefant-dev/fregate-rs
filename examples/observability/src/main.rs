use std::net::Ipv4Addr;
use std::sync::Arc;

use fregate::axum::routing::get;
use fregate::axum::Router;
use fregate::{init_tracing, AlwaysHealthy, Application};

#[tokio::main]
async fn main() {
    init_tracing();

    Application::new_with_health(Arc::new(AlwaysHealthy::default()))
        .rest_router(Router::new().route("/", get(handler)))
        .ip_addr(Ipv4Addr::new(0, 0, 0, 0))
        .port(8005u16)
        .run()
        .await
        .unwrap();
}

async fn handler() -> &'static str {
    "Hello, World!"
}
