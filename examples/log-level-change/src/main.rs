use fregate::logging::get_trace_filter;
use fregate::tokio;
use fregate::tracing::trace_span;
use fregate::{
    axum::{routing::get, Router},
    bootstrap,
    logging::get_log_filter,
    Application, Empty,
};
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

    let config = bootstrap::<Empty, _>([]).unwrap();

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(10)).await;

        let trace_span = trace_span!("This won't be sent by default");
        drop(trace_span);

        let log_filter_reloader = get_log_filter().unwrap();

        log_filter_reloader
            .modify(|filter| *filter = EnvFilter::from_str("trace").unwrap())
            .unwrap();

        let trace_filter_reloader = get_trace_filter().unwrap();
        trace_filter_reloader
            .modify(|filter| *filter = EnvFilter::from_str("trace").unwrap())
            .unwrap();

        let trace_span = trace_span!("Will be sent after reload");
        drop(trace_span);
    });

    let rest = Router::new().route("/", get(handler));

    Application::new(&config)
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
