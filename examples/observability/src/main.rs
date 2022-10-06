use fregate::logging::init_tracing_from_config;
use fregate::{
    axum::{routing::get, Router},
    middleware::http_trace_layer,
    tokio, Application, ApplicationConfig, TracingConfig,
};

#[tokio::main]
async fn main() {
    let conf = ApplicationConfig::default();
    let trace_conf = TracingConfig::default();

    init_tracing_from_config(trace_conf).unwrap();

    let rest = Router::new()
        .route("/", get(handler))
        .layer(http_trace_layer());

    Application::new(conf).router(rest).serve().await.unwrap();
}

async fn handler() -> &'static str {
    "Hello, World!"
}

/*
    curl http://0.0.0.0:8000
*/
