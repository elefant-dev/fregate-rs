use fregate::logging::init_tracing_from_config;
use fregate::middleware::http_trace_layer;
use fregate::{
    axum::{routing::get, Router},
    logging::get_handle_log_layer,
    Application,
};
use fregate::{tokio, ApplicationConfig, TracingConfig};
use std::str::FromStr;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

// Change log level after 10 seconds.
// Default log level is INFO
// Will be changed to TRACE
#[tokio::main]
async fn main() {
    let conf = ApplicationConfig::default();
    let trace_conf = TracingConfig::default();

    init_tracing_from_config(trace_conf).unwrap();

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(10)).await;
        let log_filter_reloader = get_handle_log_layer().unwrap();

        log_filter_reloader
            .modify(|filter| *filter.filter_mut() = EnvFilter::from_str("trace").unwrap())
            .unwrap()
    });

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
