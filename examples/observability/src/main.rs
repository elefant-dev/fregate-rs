use axum::routing::get;
use axum::Router;
use std::net::Ipv4Addr;

use fregate::{Application, DefaultHealth};

#[tokio::main]
async fn main() {
    let app = Application::builder::<DefaultHealth>()
        .init_metrics()
        .init_tracing()
        .set_rest_routes(Router::new().route("/", get(handler)))
        .set_address(Ipv4Addr::new(0, 0, 0, 0))
        .set_port(8000u16)
        .build();

    app.run().await.unwrap();
}

async fn handler() -> &'static str {
    "Hello, World!"
}
