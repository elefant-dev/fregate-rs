use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle, PrometheusRecorder};
use once_cell::sync::Lazy;

pub(crate) static RECORDER: Lazy<PrometheusRecorder> =
    Lazy::new(|| PrometheusBuilder::new().build_recorder());

pub(crate) static HANDLE: Lazy<PrometheusHandle> = Lazy::new(|| RECORDER.handle());
