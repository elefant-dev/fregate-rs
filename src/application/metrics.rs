#![allow(dead_code)]

use crate::Result;
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
pub fn init_metrics() -> Result<()> {
    Ok(metrics::set_recorder(&*RECORDER)?)
}

/*
use metrics::{register_counter, register_histogram, Unit};
fn register_metrics() {
    register_counter!("http_requests_total", "Incoming Requests");
    register_counter!("http_requests", "Incoming Requests");
    register_histogram!("http_response_time", Unit::Seconds, "Response Times");
}
*/
