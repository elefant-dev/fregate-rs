use metrics::{describe_counter, describe_histogram, register_counter, register_histogram, Unit};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle, PrometheusRecorder};
use once_cell::sync::Lazy;

static RECORDER: Lazy<PrometheusRecorder> = Lazy::new(|| PrometheusBuilder::new().build_recorder());
static HANDLE: Lazy<PrometheusHandle> = Lazy::new(|| RECORDER.handle());

/// Return rendered metrics
pub fn get_metrics() -> String {
    HANDLE.render()
}

/// Initialise PrometheusRecorder
///
/// Panic if failed to initialise
#[allow(clippy::expect_used)]
pub fn init_metrics() {
    metrics::set_recorder(&*RECORDER).expect("telemetry: Can't set recorder.");
    register_metrics();
}

fn register_metrics() {
    describe_counter!("http_requests_total", "Incoming Requests");
    register_counter!("http_requests_total");
    describe_counter!("http_requests", "Incoming Requests");
    register_counter!("http_requests");
    describe_histogram!("http_response_time", Unit::Seconds, "Response Times");
    register_histogram!("http_response_time");
}
