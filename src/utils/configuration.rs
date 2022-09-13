use crate::{init_tracing, DeserializeExt, LogLayerReload, TraceLayerReload};
use config::{builder::DefaultState, ConfigBuilder, ConfigError, Environment, File, FileFormat};
use serde::{
    de::{DeserializeOwned, Error},
    Deserialize, Deserializer,
};
use serde_json::{from_value, Value};
use std::sync::atomic::AtomicBool;
use std::{fmt::Debug, fmt::Formatter, marker::PhantomData, net::IpAddr, sync::atomic::Ordering};

const HOST_PTR: &str = "/host";
const PORT_PTR: &str = "/port";
const LOG_LEVEL_PTR: &str = "/log/level";
const TRACE_LEVEL_PTR: &str = "/trace/level";
const SERVICE_NAME_PTR: &str = "/service/name";
const TRACES_ENDPOINT_PTR: &str = "/exporter/otlp/traces/endpoint";
const DEFAULT_CONFIG: &str = include_str!("../resources/default_conf.toml");
const DEFAULT_SEPARATOR: &str = "_";

static CONFIG_IS_READ: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Deserialize)]
pub struct Empty {}

/// Make sure you build only one AppConfig for server, trying to build more then 1 will cause panic
#[derive(Debug)]
pub struct AppConfig<T> {
    pub host: IpAddr,
    pub port: u16,
    pub logger: LoggerConfig,
    pub init_tracing: bool,
    pub private: T,
}

pub struct LoggerConfig {
    pub log_level: String,
    pub trace_level: String,
    pub service_name: String,
    pub traces_endpoint: Option<String>,
    pub log_filter_reloader: Option<LogLayerReload>,
    pub traces_filter_reloader: Option<TraceLayerReload>,
}

impl Debug for LoggerConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoggerConfig")
            .field("log_level", &self.log_level)
            .field("trace_level", &self.trace_level)
            .field("service_name", &self.service_name)
            .field("traces_endpoint", &self.traces_endpoint)
            .field("log_filter_reloader", &self.log_filter_reloader.is_some())
            .field(
                "traces_filter_reloader",
                &self.traces_filter_reloader.is_some(),
            )
            .finish()
    }
}

impl<'de> Deserialize<'de> for LoggerConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut config = Value::deserialize(deserializer)?;

        let log_level = config.pointer_and_deserialize(LOG_LEVEL_PTR)?;
        let trace_level = config.pointer_and_deserialize(TRACE_LEVEL_PTR)?;
        let service_name = config.pointer_and_deserialize(SERVICE_NAME_PTR)?;
        let traces_endpoint_ptr = config.pointer_mut(TRACES_ENDPOINT_PTR);

        let traces_endpoint = if let Some(ptr) = traces_endpoint_ptr {
            Some(from_value::<String>(ptr.take()).map_err(Error::custom)?)
        } else {
            None
        };

        Ok(LoggerConfig {
            log_level,
            trace_level,
            service_name,
            traces_endpoint,
            log_filter_reloader: None,
            traces_filter_reloader: None,
        })
    }
}

impl<'de, T> Deserialize<'de> for AppConfig<T>
where
    T: Debug + DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let config = Value::deserialize(deserializer)?;

        let host = config.pointer_and_deserialize(HOST_PTR)?;
        let port = config.pointer_and_deserialize(PORT_PTR)?;
        let logger = LoggerConfig::deserialize(&config).map_err(Error::custom)?;
        let private = T::deserialize(config).map_err(Error::custom)?;

        Ok(AppConfig::<T> {
            host,
            port,
            init_tracing: false,
            logger,
            private,
        })
    }
}

impl Default for AppConfig<Empty> {
    fn default() -> Self {
        AppConfig::builder()
            .add_default()
            .init_tracing()
            .build()
            .expect("Default config never fails")
    }
}

impl<T> AppConfig<T> {
    pub fn get_log_filter_reload(&self) -> Option<&LogLayerReload> {
        self.logger.log_filter_reloader.as_ref()
    }
}

impl<T: DeserializeOwned + Debug> AppConfig<T> {
    pub fn builder() -> AppConfigBuilder<T> {
        AppConfigBuilder::new()
    }

    pub fn default_with(file_path: &str, env_prefix: &str) -> Result<Self, ConfigError> {
        AppConfig::builder()
            .add_default()
            .init_tracing()
            .add_file(file_path)
            .add_env_prefixed(env_prefix)
            .build()
    }
}

#[derive(Debug, Default)]
pub struct AppConfigBuilder<T> {
    builder: ConfigBuilder<DefaultState>,
    init_tracing: bool,
    phantom: PhantomData<T>,
}

impl<T: DeserializeOwned + Debug> AppConfigBuilder<T> {
    pub fn new() -> Self {
        Self {
            builder: ConfigBuilder::default(),
            init_tracing: false,
            phantom: PhantomData,
        }
    }

    pub fn build(self) -> Result<AppConfig<T>, ConfigError> {
        if CONFIG_IS_READ.swap(true, Ordering::SeqCst) {
            panic!("Only one config is allowed to read")
        }

        let mut config = self.builder.build()?.try_deserialize::<AppConfig<T>>()?;
        config.init_tracing = self.init_tracing;

        if config.init_tracing {
            init_tracing(&mut config);
        }

        Ok(config)
    }

    pub fn init_tracing(mut self) -> Self {
        self.init_tracing = true;
        self
    }

    pub fn add_default(mut self) -> Self {
        self.builder = self
            .builder
            .add_source(File::from_str(DEFAULT_CONFIG, FileFormat::Toml));
        self.add_env_prefixed("OTEL")
    }

    pub fn add_file(mut self, path: &str) -> Self {
        self.builder = self.builder.add_source(File::with_name(path));
        self
    }

    pub fn add_str(mut self, str: &str, format: FileFormat) -> Self {
        self.builder = self.builder.add_source(File::from_str(str, format));
        self
    }

    pub fn add_env_prefixed(mut self, prefix: &str) -> Self {
        self.builder = self.builder.add_source(
            Environment::with_prefix(prefix)
                .try_parsing(true)
                .separator(DEFAULT_SEPARATOR),
        );
        self
    }
}
