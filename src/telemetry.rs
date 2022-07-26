use metrics::{register_counter, register_histogram, Unit};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusRecorder};
use tracing::Level;
use tracing_subscriber::filter::FromEnvError;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::EnvFilter;

lazy_static::lazy_static! {
    static ref RECORDER: PrometheusRecorder = PrometheusBuilder::new().build();
}

pub fn init() {
    // Init tracing

    let filter = tracing_filter().unwrap_or_else(|_| EnvFilter::default());

    tracing_subscriber::fmt()
        .json()
        .with_timer(UtcTime::rfc_3339())
        .flatten_event(false)
        .with_target(true)
        .with_current_span(true)
        // Use the filter we built above to determine which traces to record.
        .with_env_filter(filter)
        // Record an event when each span closes. This can be used to time our
        // routes' durations!
        .with_span_events(FmtSpan::ACTIVE)
        .with_max_level(Level::DEBUG)
        .init();

    // Capture the span context in which the program panicked
    std::panic::set_hook(Box::new(|panic| {
        // If the panic has a source location, record it as structured fields.
        if let Some(location) = panic.location() {
            tracing::error!(
                message = %panic,
                panic.file = location.file(),
                panic.line = location.line(),
                panic.column = location.column(),
            );
        } else {
            tracing::error!(message = %panic);
        }
    }));

    // Init metrics
   //  metrics::set_recorder(&*RECORDER).unwrap();

    register_metrics()
}

fn tracing_filter() -> Result<EnvFilter, FromEnvError> {
    let filter = EnvFilter::try_from_default_env()?
        .add_directive(Level::INFO.into())
        .add_directive("hyper=info".parse()?)
        .add_directive("tower_http=trace".parse()?)
        .add_directive("idunn=trace".parse()?);

    Ok(filter)
}

pub fn get_metrics() -> String {
    RECORDER.handle().render()
}

fn register_metrics() -> () {
    // register_counter!("http_requests_total", "Incoming Requests");
    // register_counter!("http_requests", "Incoming Requests");
//    register_histogram!("http_response_time", Unit::Seconds, "Response Times");
}
