use crate::extensions::DeserializeExt;
use crate::observability::HeadersFilter;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use serde_json::{from_value, Value};
use std::time::Duration;

const SERVER_METRICS_UPDATE_INTERVAL_PTR: &str = "/server/metrics/update_interval";
const LOG_LEVEL_PTR: &str = "/log/level";
const LOG_MSG_LENGTH_PTR: &str = "/log/msg/length";
const LOGGING_FILE_PTR: &str = "/logging/file";
const LOGGING_PATH_PTR: &str = "/logging/path";
const LOGGING_INTERVAL_PTR: &str = "/logging/interval";
const LOGGING_LIMIT_PTR: &str = "/logging/limit";
const LOGGING_MAX_AGE_PTR: &str = "/logging/max/age";
const LOGGING_MAX_COUNT_PTR: &str = "/logging/max/count";
const LOGGING_ENABLE_ZIP_PTR: &str = "/logging/enable/zip";
const BUFFERED_LINES_LIMIT_PTR: &str = "/buffered/lines/limit";
const TRACE_LEVEL_PTR: &str = "/trace/level";
const SERVICE_NAME_PTR: &str = "/service/name";
const COMPONENT_NAME_PTR: &str = "/component/name";
const COMPONENT_VERSION_PTR: &str = "/component/version";
const TRACES_ENDPOINT_PTR: &str = "/exporter/otlp/traces/endpoint";
const CGROUP_METRICS_PTR: &str = "/cgroup/metrics";
const HEADERS_PTR: &str = "/headers";

/// Configuration for logs and traces
#[derive(Debug, Clone, Default)]
pub struct ObservabilityConfig {
    /// service name to be used in logs
    pub service_name: String,
    /// component name to be used in logs and traces
    pub component_name: String,
    /// component version
    pub version: String,
    /// logger configuration
    pub logger_config: LoggerConfig,
    /// if it set true then metrics will be supplied from cgroup v2.
    pub cgroup_metrics: bool,
    /// metrics update interval
    pub metrics_update_interval: Duration,
    /// trace level read to string and later parsed into EnvFilter
    pub trace_level: String,
    /// configures [`tracing_opentelemetry::layer`] endpoint for sending traces.
    pub traces_endpoint: Option<String>,
}

/// Configuration for Logs
#[derive(Debug, Clone, Default)]
pub struct LoggerConfig {
    /// log level read to string and later parsed into EnvFilter
    pub log_level: String,
    /// path where to write logs, if set logs will be written to file
    pub logging_path: Option<String>,
    /// log file prefix, if written to file all log files will be with this prefix.\
    /// by default `component_name`  from [`ObservabilityConfig`] is used
    pub logging_file: Option<String>,
    /// Maximum message field length, if set: message field will be cut if len() exceed this limit
    pub msg_length: Option<usize>,
    /// Sets limit for [`tracing_appender::non_blocking::NonBlocking`]
    pub buffered_lines_limit: Option<usize>,
    /// interval to split file into chunks with fixed interval
    pub logging_interval: Option<Duration>,
    /// file size limit in bytes
    pub logging_limit: Option<usize>,
    /// maximum duration files kept in seconds
    pub logging_max_age: Option<Duration>,
    /// maximum number of files kept
    pub logging_max_count: Option<usize>,
    /// enable files zipping
    pub logging_enable_zip: bool,
    /// initialize [`crate::observability::HEADERS_FILTER`] static variable in [`crate::bootstrap()`] or [`crate::observability::init_tracing()`] fn.
    pub headers_filter: Option<HeadersFilter>,
}

impl<'de> Deserialize<'de> for ObservabilityConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut config = Value::deserialize(deserializer)?;

        let trace_level = config.pointer_and_deserialize(TRACE_LEVEL_PTR)?;
        let service_name = config.pointer_and_deserialize(SERVICE_NAME_PTR)?;
        let component_name = config.pointer_and_deserialize(COMPONENT_NAME_PTR)?;

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
        let logger_config = LoggerConfig::deserialize(config).map_err(Error::custom)?;

        Ok(ObservabilityConfig {
            version,
            trace_level,
            service_name,
            component_name,
            traces_endpoint,
            metrics_update_interval: Duration::from_millis(metrics_update_interval),
            cgroup_metrics,
            logger_config,
        })
    }
}

impl<'de> Deserialize<'de> for LoggerConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let config = Value::deserialize(deserializer)?;

        let log_level = config.pointer_and_deserialize(LOG_LEVEL_PTR)?;
        let msg_length = config
            .pointer_and_deserialize::<_, D::Error>(LOG_MSG_LENGTH_PTR)
            .ok();
        let buffered_lines_limit = config
            .pointer_and_deserialize::<_, D::Error>(BUFFERED_LINES_LIMIT_PTR)
            .ok();
        let logging_file = config
            .pointer_and_deserialize::<_, D::Error>(LOGGING_FILE_PTR)
            .unwrap_or_default();
        let logging_path = config
            .pointer_and_deserialize::<_, D::Error>(LOGGING_PATH_PTR)
            .unwrap_or_default();
        let headers_filter: Option<HeadersFilter> = config
            .pointer_and_deserialize::<_, D::Error>(HEADERS_PTR)
            .ok();

        let logging_interval = config
            .pointer_and_deserialize::<u64, D::Error>(LOGGING_INTERVAL_PTR)
            .ok()
            .map(Duration::from_secs);
        let logging_limit = config
            .pointer_and_deserialize::<_, D::Error>(LOGGING_LIMIT_PTR)
            .ok();
        let logging_max_age = config
            .pointer_and_deserialize::<u64, D::Error>(LOGGING_MAX_AGE_PTR)
            .ok()
            .map(Duration::from_secs);
        let logging_max_count = config
            .pointer_and_deserialize::<_, D::Error>(LOGGING_MAX_COUNT_PTR)
            .ok();
        let logging_enable_zip = config
            .pointer_and_deserialize::<_, D::Error>(LOGGING_ENABLE_ZIP_PTR)
            .unwrap_or_default();

        Ok(LoggerConfig {
            log_level,
            msg_length,
            buffered_lines_limit,
            logging_file,
            logging_path,
            logging_interval,
            logging_limit,
            logging_max_age,
            logging_max_count,
            logging_enable_zip,
            headers_filter,
        })
    }
}
