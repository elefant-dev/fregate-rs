#[cfg(feature = "tokio-metrics")]
pub mod tokio_metrics;

use crate::error::Result;
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

    #[cfg(feature = "tokio-metrics")]
    tokio_metrics::register_metrics();

    Ok(())
}
