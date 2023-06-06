use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle, PrometheusRecorder};
use once_cell::sync::OnceCell;

pub(crate) fn get_recorder() -> &'static PrometheusRecorder {
    static RECORDER: OnceCell<PrometheusRecorder> = OnceCell::new();

    RECORDER.get_or_init(|| PrometheusBuilder::new().build_recorder())
}

pub(crate) fn get_handle() -> &'static PrometheusHandle {
    static HANDLE: OnceCell<PrometheusHandle> = OnceCell::new();

    HANDLE.get_or_init(|| get_recorder().handle())
}
