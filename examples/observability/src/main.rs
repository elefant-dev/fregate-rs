use fregate::{
    axum::{routing::get, Router},
    init_tracing, AlwaysReadyAndAlive, Application,
};
use std::net::Ipv4Addr;

#[tokio::main]
async fn main() {
    init_tracing();

    Application::new_with_health(AlwaysReadyAndAlive::default())
        .rest_router(Router::new().route("/", get(handler)))
        .host(Ipv4Addr::new(0, 0, 0, 0))
        .port(8005u16)
        .run()
        .await
        .unwrap();
}

async fn handler() -> &'static str {
    "Hello, World!"
}
