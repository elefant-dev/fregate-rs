use fregate::{
    axum::{routing::get, Router},
    init_tracing, AlwaysReadyAndAlive, AppConfig, Application,
};

#[tokio::main]
async fn main() {
    init_tracing();

    let config = AppConfig::default();
    let health = AlwaysReadyAndAlive::default();

    Application::new_with_health(config)
        .health_indicator(health)
        .rest_router(Router::new().route("/", get(handler)))
        .run()
        .await
        .unwrap();
}

async fn handler() -> &'static str {
    "Hello, World!"
}
