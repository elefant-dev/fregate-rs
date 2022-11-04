#[cfg(feature = "tokio-metrics")]
mod tokio_metrics;

use crate::error::Result;
use crate::AppConfig;
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
#[allow(unused_param)]
pub fn init_metrics<T>(config: &AppConfig<T>) -> Result<()> {
    register_metrics();
    metrics::set_recorder(&*RECORDER)?;
    #[cfg(feature = "tokio-metrics")]
    tokio_metrics::init_tokio_metrics_task(config);

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
