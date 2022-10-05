#![allow(dead_code)]

use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle, PrometheusRecorder};
use once_cell::sync::Lazy;

static RECORDER: Lazy<PrometheusRecorder> = Lazy::new(|| PrometheusBuilder::new().build_recorder());
static HANDLE: Lazy<PrometheusHandle> = Lazy::new(|| RECORDER.handle());

// TODO(kos): Don't make compiler to inline, unless explicitly proven that the
//            concrete inline removes bottle-neck. Aggressive manual inlining
//            may increase compilation time and worsen the performance. Let the
//            compiler does his job.
//            https://matklad.github.io/2021/07/09/inline-in-rust.html
#[inline(always)]
/// Return rendered metrics
pub fn get_metrics() -> String {
    HANDLE.render()
}

#[inline(always)]
#[allow(clippy::expect_used)]
/// Initialise PrometheusRecorder
///
/// Panic if failed to initialise
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
