use fregate::{
    axum::{routing::get, Router},
    http_trace_layer, init_tracing, AlwaysReadyAndAlive, AppConfig, Application,
};

#[tokio::main]
async fn main() {
    init_tracing();

    let config = AppConfig::default();
    let health = AlwaysReadyAndAlive::default();

    let rest = Router::new()
        .route("/", get(handler))
        .layer(http_trace_layer());

    Application::new(&config)
        .health_indicator(health)
        .router(rest)
        .serve()
        .await
        .unwrap();
}

async fn handler() -> &'static str {
    "Hello, World!"
}

/*
    curl http://0.0.0.0:8000
*/
