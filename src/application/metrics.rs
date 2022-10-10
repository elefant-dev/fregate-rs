use metrics::{describe_counter, describe_histogram, register_counter, register_histogram, Unit};
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

/// Initialise PrometheusRecorder
///
/// Panic if failed to initialise
#[allow(clippy::expect_used)]
pub fn init_metrics() {
    metrics::set_recorder(&*RECORDER).expect("telemetry: Can't set recorder.");
    register_metrics();
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
}
