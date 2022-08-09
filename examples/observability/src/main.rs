use fregate::{
    axum::{routing::get, Router},
    init_logging, AlwaysReadyAndAlive, AppConfig, Application,
};

#[tokio::main]
async fn main() {
    init_logging();

    let config = AppConfig::default();
    let health = AlwaysReadyAndAlive::default();

    Application::new_with_health(config)
        .health_indicator(health)
        .rest_router(Router::new().route("/", get(handler)))
        .serve()
        .await
        .unwrap();
}

async fn handler() -> &'static str {
    "Hello, World!"
}
