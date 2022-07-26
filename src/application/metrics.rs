#[cfg(feature = "tokio-metrics")]
pub mod tokio_metrics;

use crate::error::Result;
use metrics::{describe_counter, describe_histogram, register_counter, register_histogram, Unit};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle, PrometheusRecorder};
use once_cell::sync::Lazy;

static RECORDER: Lazy<PrometheusRecorder> = Lazy::new(|| PrometheusBuilder::new().build_recorder());
static HANDLE: Lazy<PrometheusHandle> = Lazy::new(|| RECORDER.handle());

/// Return rendered metrics
pub fn get_metrics(callback: Option<&(dyn Fn() + Send + Sync + 'static)>) -> String {
    if let Some(callback) = callback {
        callback();
    }

    HANDLE.render()
}

/// Initialise PrometheusRecorder
pub fn init_metrics() -> Result<()> {
    metrics::set_recorder(&*RECORDER)?;
    register_metrics();

    Ok(())
}

fn register_metrics() {
    describe_counter!(
        "traffic_count_total",
        "The accumulated counter for number of messages."
    );
    register_counter!("traffic_count_total");
    describe_counter!(
        "traffic_sum_total",
        "The accumulated counter used for calculating traffic."
    );
    register_counter!("traffic_sum_total");
    describe_histogram!(
        "processing_duration_seconds_sum_total",
        Unit::Seconds,
        "Response Times"
    );
    register_histogram!("processing_duration_seconds_sum_total");

    #[cfg(feature = "tokio-metrics")]
    tokio_metrics::register_metrics();
}
