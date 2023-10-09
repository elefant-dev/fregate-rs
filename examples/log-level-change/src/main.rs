use fregate::observability::{LOG_LAYER_HANDLE, OTLP_LAYER_HANDLE};
use fregate::tracing::trace_span;
use fregate::{
    axum::{routing::get, Router},
    bootstrap, Application,
};
use fregate::{tokio, AppConfig};
use std::str::FromStr;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

// Change log level after 10 seconds.
// Default log level is INFO
// Will be changed to TRACE
#[tokio::main]
async fn main() {
    std::env::set_var("OTEL_COMPONENT_NAME", "server");
    std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "http://0.0.0.0:4317");

    let config: AppConfig = bootstrap([]).unwrap();

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(10)).await;

        let trace_span = trace_span!("This won't be sent by default");
        drop(trace_span);

        let log_filter_reloader = LOG_LAYER_HANDLE.get().unwrap();

        log_filter_reloader
            .modify(|filter| *filter = EnvFilter::from_str("trace").unwrap())
            .unwrap();

        let trace_filter_reloader = OTLP_LAYER_HANDLE.get().unwrap();
        trace_filter_reloader
            .modify(|filter| *filter = EnvFilter::from_str("trace").unwrap())
            .unwrap();

        let trace_span = trace_span!("Will be sent after reload");
        drop(trace_span);
    });

    let rest = Router::new().route("/", get(handler));

    Application::new(config).router(rest).serve().await.unwrap();
}

async fn handler() -> &'static str {
    "Hello, World!"
}

/*
    curl http://0.0.0.0:8000
*/
