use crate::extensions::DeserializeExt;
use crate::observability::HeadersFilter;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use serde_json::{from_value, Value};

const SERVER_METRICS_UPDATE_INTERVAL_PTR: &str = "/server/metrics/update_interval";
const LOG_LEVEL_PTR: &str = "/log/level";
const LOG_MSG_LENGTH_PTR: &str = "/log/msg/length";
const LOGGING_FILE_PTR: &str = "/logging/file";
const LOGGING_PATH_PTR: &str = "/logging/path";
const BUFFERED_LINES_LIMIT_PTR: &str = "/buffered/lines/limit";
const TRACE_LEVEL_PTR: &str = "/trace/level";
const SERVICE_NAME_PTR: &str = "/service/name";
const COMPONENT_NAME_PTR: &str = "/component/name";
const COMPONENT_VERSION_PTR: &str = "/component/version";
const TRACES_ENDPOINT_PTR: &str = "/exporter/otlp/traces/endpoint";
const CGROUP_METRICS_PTR: &str = "/cgroup/metrics";
const HEADERS_PTR: &str = "/headers";

/// configuration for logs and traces
#[derive(Debug, Clone, Default)]
pub struct ObservabilityConfig {
    /// log level read to string and later parsed into EnvFilter
    pub log_level: String,
    /// path where to write logs, if set logs will be written to file
    pub logging_path: Option<String>,
    /// log file prefix, if written to file all log files will be with this prefix.\
    /// by default `component_name` is used
    pub logging_file: Option<String>,
    /// Maximum message field length, if set: message field will be cut if len() exceed this limit
    pub msg_length: Option<usize>,
    /// Sets limit for [`tracing_appender::non_blocking::NonBlocking`]
    pub buffered_lines_limit: Option<usize>,
    /// trace level read to string and later parsed into EnvFilter
    pub trace_level: String,
    /// service name to be used in logs
    pub service_name: String,
    /// component name to be used in logs and traces
    pub component_name: String,
    /// component version
    pub version: String,
    /// if it set true then metrics will be supplied from cgroup v2.
    pub cgroup_metrics: bool,
    /// metrics update interval
    pub metrics_update_interval: std::time::Duration,
    /// configures [`tracing_opentelemetry::layer`] endpoint for sending traces.
    pub traces_endpoint: Option<String>,
    /// initialize [`crate::observability::HEADERS_FILTER`] static variable in [`crate::bootstrap()`] or [`crate::observability::init_tracing()`] fn.
    pub headers_filter: Option<HeadersFilter>,
}

impl<'de> Deserialize<'de> for ObservabilityConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut config = Value::deserialize(deserializer)?;

        let log_level = config.pointer_and_deserialize(LOG_LEVEL_PTR)?;
        let trace_level = config.pointer_and_deserialize(TRACE_LEVEL_PTR)?;
        let service_name = config.pointer_and_deserialize(SERVICE_NAME_PTR)?;
        let component_name = config.pointer_and_deserialize(COMPONENT_NAME_PTR)?;
        let logging_file = config
            .pointer_and_deserialize::<_, D::Error>(LOGGING_FILE_PTR)
            .unwrap_or_default();
        let logging_path = config
            .pointer_and_deserialize::<_, D::Error>(LOGGING_PATH_PTR)
            .unwrap_or_default();
        let cgroup_metrics = config
            .pointer_and_deserialize::<_, D::Error>(CGROUP_METRICS_PTR)
            .unwrap_or_default();
        let version = config.pointer_and_deserialize(COMPONENT_VERSION_PTR)?;
        let traces_endpoint = config
            .pointer_mut(TRACES_ENDPOINT_PTR)
            .map(Value::take)
            .map(from_value::<String>)
            .transpose()
            .map_err(D::Error::custom)?;
        let metrics_update_interval =
            config.pointer_and_deserialize::<u64, D::Error>(SERVER_METRICS_UPDATE_INTERVAL_PTR)?;
        let msg_length = config
            .pointer_and_deserialize::<_, D::Error>(LOG_MSG_LENGTH_PTR)
            .ok();
        let buffered_lines_limit = config
            .pointer_and_deserialize::<_, D::Error>(BUFFERED_LINES_LIMIT_PTR)
            .ok();
        let headers_filter: Option<HeadersFilter> = config
            .pointer_and_deserialize::<_, D::Error>(HEADERS_PTR)
            .ok();

        Ok(ObservabilityConfig {
            log_level,
            logging_path,
            logging_file,
            msg_length,
            version,
            trace_level,
            service_name,
            component_name,
            traces_endpoint,
            buffered_lines_limit,
            headers_filter,
            metrics_update_interval: std::time::Duration::from_millis(metrics_update_interval),
            cgroup_metrics,
        })
    }
}
