use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle, PrometheusRecorder};
use std::sync::OnceLock;

pub(crate) fn get_recorder() -> &'static PrometheusRecorder {
    static RECORDER: OnceLock<PrometheusRecorder> = OnceLock::new();

    RECORDER.get_or_init(|| PrometheusBuilder::new().build_recorder())
}

pub(crate) fn get_handle() -> &'static PrometheusHandle {
    static HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

    HANDLE.get_or_init(|| get_recorder().handle())
}
