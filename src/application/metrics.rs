use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle, PrometheusRecorder};
use metrics_util::MetricKindMask;
use once_cell::sync::Lazy;
use std::net::ToSocketAddrs;

static RECORDER: Lazy<PrometheusRecorder> = Lazy::new(|| PrometheusBuilder::new().build_recorder());
static HANDLE: Lazy<PrometheusHandle> = Lazy::new(|| RECORDER.handle());

#[inline(always)]
pub fn get_metrics() -> String {
    HANDLE.render()
}

#[inline(always)]
pub fn init_metrics(metrics_endpoint: &Option<String>) {
    //metrics::set_recorder(&*RECORDER).expect("telemetry: Can't set recorder.");

    let socket = match metrics_endpoint {
        Some(socket) => socket
            .to_socket_addrs()
            .expect("Unable to parse socket address")
            .next()
            .expect("Unable to parse socket address"),
        None => return,
    };

    PrometheusBuilder::new()
        .with_http_listener(socket)
        .idle_timeout(
            MetricKindMask::COUNTER | MetricKindMask::HISTOGRAM,
            Some(std::time::Duration::from_secs(10)),
        )
        .install()
        .expect("failed to install Prometheus recorder");

    register_metrics();
}

use metrics::{describe_counter, describe_histogram, register_counter, register_histogram, Unit};
fn register_metrics() {
    describe_counter!("http_requests_total", "Incoming Requests");
    register_counter!("http_requests_total");
    describe_counter!("http_requests", "Incoming Requests");
    register_counter!("http_requests");
    describe_histogram!("http_response_time", Unit::Seconds, "Response Times");
    register_histogram!("http_response_time");
}
