pub mod headers;
#[cfg(feature = "tls")]
mod tls;

use crate::{error::Result, extensions::DeserializeExt};
use config::{builder::DefaultState, ConfigBuilder, Environment, File, FileFormat};
use headers::HeadersFilter;
use serde::{
    de::{DeserializeOwned, Error},
    Deserialize, Deserializer,
};
use serde_json::{from_value, Value};
use std::marker::PhantomData;
use std::{fmt::Debug, net::IpAddr};
use tracing_appender::non_blocking::WorkerGuard;

const HOST_PTR: &str = "/host";
const PORT_PTR: &str = "/port";
#[cfg(feature = "tokio-metrics")]
const SERVER_METRICS_UPDATE_INTERVAL_PTR: &str = "/server/metrics/update_interval";
const LOG_LEVEL_PTR: &str = "/log/level";
const LOG_MSG_LENGTH_PTR: &str = "/log/msg/length";
const BUFFERED_LINES_LIMIT_PTR: &str = "/buffered/lines/limit";
const TRACE_LEVEL_PTR: &str = "/trace/level";
const SERVICE_NAME_PTR: &str = "/service/name";
const COMPONENT_NAME_PTR: &str = "/component/name";
const COMPONENT_VERSION_PTR: &str = "/component/version";
const TRACES_ENDPOINT_PTR: &str = "/exporter/otlp/traces/endpoint";
const HEADERS_PTR: &str = "/headers";

const DEFAULT_CONFIG: &str = include_str!("../resources/default_conf.toml");
const DEFAULT_SEPARATOR: &str = "_";

/// Enum to specify configuration source type:
#[derive(Clone, Debug)]
pub enum ConfigSource<'a> {
    /// Load from string
    String(&'a str, FileFormat),
    /// Read file by given path
    File(&'a str),
    /// Read environment variables with specified prefix
    EnvPrefix(&'a str),
}

/// Used as dummy structure to read [`AppConfig`] without private field
#[derive(Deserialize, Debug, PartialEq, Eq, Copy, Clone)]
pub struct Empty {}

/// AppConfig reads and saves application configuration from different sources
#[derive(Debug)]
pub struct AppConfig<T> {
    /// host address where to start Application
    pub host: IpAddr,
    /// port
    pub port: u16,
    /// configuration for logs and traces
    pub logger: LoggerConfig,
    #[cfg(feature = "tls")]
    /// TLS configuration parameters
    pub tls: tls::TlsConfigurationVariables,
    /// field for each application specific configuration
    pub private: T,
    /// Why it is here read more: [`https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/struct.WorkerGuard.html`]
    pub(crate) worker_guard: Option<WorkerGuard>,
}

/// configuration for logs and traces
#[derive(Debug)]
pub struct LoggerConfig {
    /// log level read to string and later parsed into EnvFilter
    pub log_level: String,
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
    /// Tokio metrics update interval
    #[cfg(feature = "tokio-metrics")]
    pub metrics_update_interval: std::time::Duration,
    /// configures [`tracing_opentelemetry::layer`] endpoint for sending traces.
    pub traces_endpoint: Option<String>,
    /// initialize [`crate::logging::HEADER_FILTER`] static variable in [`bootstrap`] or [`init_tracing`] fn.
    pub header_filter: Option<HeadersFilter>,
}

impl<'de> Deserialize<'de> for LoggerConfig {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut config = Value::deserialize(deserializer)?;

        let log_level = config.pointer_and_deserialize(LOG_LEVEL_PTR)?;
        let trace_level = config.pointer_and_deserialize(TRACE_LEVEL_PTR)?;
        let service_name = config.pointer_and_deserialize(SERVICE_NAME_PTR)?;
        let component_name = config.pointer_and_deserialize(COMPONENT_NAME_PTR)?;
        let version = config.pointer_and_deserialize(COMPONENT_VERSION_PTR)?;
        let traces_endpoint = config
            .pointer_mut(TRACES_ENDPOINT_PTR)
            .map(Value::take)
            .map(from_value::<String>)
            .transpose()
            .map_err(Error::custom)?;
        #[cfg(feature = "tokio-metrics")]
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

        Ok(LoggerConfig {
            log_level,
            msg_length,
            version,
            trace_level,
            service_name,
            component_name,
            traces_endpoint,
            buffered_lines_limit,
            header_filter: headers_filter,
            #[cfg(feature = "tokio-metrics")]
            metrics_update_interval: std::time::Duration::from_millis(metrics_update_interval),
        })
    }
}

impl<'de, T> Deserialize<'de> for AppConfig<T>
where
    T: Debug + DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let config = Value::deserialize(deserializer)?;

        let host = config.pointer_and_deserialize(HOST_PTR)?;
        let port = config.pointer_and_deserialize(PORT_PTR)?;
        let logger = LoggerConfig::deserialize(&config).map_err(Error::custom)?;
        #[cfg(feature = "tls")]
        let tls = tls::TlsConfigurationVariables::deserialize(&config).map_err(Error::custom)?;
        let private = T::deserialize(config).map_err(Error::custom)?;

        Ok(AppConfig::<T> {
            host,
            port,
            logger,
            #[cfg(feature = "tls")]
            tls,
            private,
            worker_guard: None,
        })
    }
}

impl Default for AppConfig<Empty> {
    #[allow(clippy::expect_used)]
    fn default() -> Self {
        AppConfig::builder()
            .add_default()
            .add_env_prefixed("OTEL")
            .build()
            .expect("Default config never fails")
    }
}

impl<T> AppConfig<T> {
    /// Creates [`AppConfigBuilder`] to add different sources to config
    pub fn builder() -> AppConfigBuilder<T> {
        AppConfigBuilder::new()
    }

    /// Load file by given path and add environment variables with given prefix in addition to default config
    ///
    /// Environment variables have highet priority then file and then default configuration
    pub fn default_with(file_path: &str, env_prefix: &str) -> Result<Self>
    where
        T: Debug + DeserializeOwned,
    {
        AppConfig::builder()
            .add_default()
            .add_env_prefixed("OTEL")
            .add_file(file_path)
            .add_env_prefixed(env_prefix)
            .build()
    }

    /// Load configuration from provided container with [`ConfigSource`] which override default config.
    pub fn load_from<'a, S>(sources: S) -> Result<Self>
    where
        T: Debug + DeserializeOwned,
        S: IntoIterator<Item = ConfigSource<'a>>,
    {
        let mut config_builder = AppConfig::<T>::builder()
            .add_default()
            .add_env_prefixed("OTEL");

        for source in sources {
            config_builder = match source {
                ConfigSource::String(str, format) => config_builder.add_str(str, format),
                ConfigSource::File(path) => config_builder.add_file(path),
                ConfigSource::EnvPrefix(prefix) => config_builder.add_env_prefixed(prefix),
            };
        }

        config_builder.build()
    }
}

/// AppConfig builder to set up multiple sources
#[derive(Debug, Default)]
pub struct AppConfigBuilder<T> {
    builder: ConfigBuilder<DefaultState>,
    phantom: PhantomData<T>,
}

impl<T> AppConfigBuilder<T> {
    /// Creates new [`AppConfigBuilder`]
    pub fn new() -> Self {
        Self {
            builder: ConfigBuilder::default(),
            phantom: PhantomData,
        }
    }

    /// Reads all registered sources
    pub fn build(self) -> Result<AppConfig<T>>
    where
        T: Debug + DeserializeOwned,
    {
        Ok(self.builder.build()?.try_deserialize::<AppConfig<T>>()?)
    }

    /// Add default config
    pub fn add_default(mut self) -> Self {
        self.builder = self
            .builder
            .add_source(File::from_str(DEFAULT_CONFIG, FileFormat::Toml));
        self
    }

    /// Add file
    pub fn add_file(mut self, path: &str) -> Self {
        self.builder = self.builder.add_source(File::with_name(path));
        self
    }

    /// Add string
    pub fn add_str(mut self, str: &str, format: FileFormat) -> Self {
        self.builder = self.builder.add_source(File::from_str(str, format));
        self
    }

    /// Add environment variables with specified prefix and default separator: "_"
    pub fn add_env_prefixed(mut self, prefix: &str) -> Self {
        self.builder = self.builder.add_source(
            Environment::with_prefix(prefix)
                .try_parsing(true)
                .separator(DEFAULT_SEPARATOR),
        );
        self
    }
}
