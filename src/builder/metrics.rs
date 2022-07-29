use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle, PrometheusRecorder};
use once_cell::sync::Lazy;

static RECORDER: Lazy<PrometheusRecorder> = Lazy::new(|| PrometheusBuilder::new().build_recorder());
static HANDLE: Lazy<PrometheusHandle> = Lazy::new(|| RECORDER.handle());

#[inline(always)]
pub fn get_metrics() -> String {
    HANDLE.render()
}

#[inline(always)]
pub fn init_metrics() {
    metrics::set_recorder(&*RECORDER).expect("telemetry: Can't set recorder.");
}

/*
use metrics::{register_counter, register_histogram, Unit};
fn register_metrics() {
    register_counter!("http_requests_total", "Incoming Requests");
    register_counter!("http_requests", "Incoming Requests");
    register_histogram!("http_response_time", Unit::Seconds, "Response Times");
}
*/
