use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle, PrometheusRecorder};
use metrics_util::MetricKindMask;
use once_cell::sync::Lazy;

static RECORDER: Lazy<PrometheusRecorder> = Lazy::new(|| {
    PrometheusBuilder::new()
        .idle_timeout(
            MetricKindMask::COUNTER | MetricKindMask::HISTOGRAM,
            Some(std::time::Duration::from_secs(10)),
        )
        .build_recorder()
});
static HANDLE: Lazy<PrometheusHandle> = Lazy::new(|| RECORDER.handle());

#[inline(always)]
pub fn get_metrics() -> String {
    HANDLE.render()
}

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs},
    str::FromStr,
};
#[inline(always)]
pub fn init_metrics(traces_endpoint: &str) {
    //metrics::set_recorder(&*RECORDER).expect("telemetry: Can't set recorder.");
    let socket = SocketAddr::from_str(traces_endpoint).expect("Unable to parse socket address");

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
