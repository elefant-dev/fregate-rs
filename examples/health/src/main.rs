use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use tracing::info;

use fregate::application::Application;
use fregate::health::{HealthIndicatorRef, UpHealth};

#[tokio::main]
async fn main() {
    let health = Arc::new(UpHealth::default()) as HealthIndicatorRef;

    let app = Application::builder()
        .telemetry(true)
        .port(8000u16)
        .telemetry(true)
        .health(Some(health))
        .rest_router(Router::new().route("/", get(handler)))
        .build();

    app.run().await.unwrap();
}

async fn handler() -> &'static str {
    "Hello, World!"
}
