use crate::{DeserializeAndLog, DeserializeExt, Result};
use config::{builder::DefaultState, ConfigBuilder, Environment, File, FileFormat};
use serde::{
    de::{DeserializeOwned, Error},
    Deserialize, Deserializer,
};
use serde_json::{from_value, Value};
use std::{fmt::Debug, marker::PhantomData, net::IpAddr};

const HOST_PTR: &str = "/host";
const PORT_PTR: &str = "/port";
const LOG_LEVEL_PTR: &str = "/log/level";
const SERVICE_NAME_PTR: &str = "/service/name";
const TRACES_ENDPOINT_PTR: &str = "/exporter/otlp/traces/endpoint";
const DEFAULT_CONFIG: &str = include_str!("../resources/default_conf.toml");
const DEFAULT_SEPARATOR: &str = "_";

#[derive(Debug, Deserialize)]
pub struct Empty {}

#[derive(Debug)]
pub struct AppConfig<T> {
    pub host: IpAddr,
    pub port: u16,
    pub logger: LoggerConfig,
    pub private: T,
}

#[derive(Debug)]
pub struct LoggerConfig {
    pub log_level: String,
    pub service_name: String,
    pub traces_endpoint: Option<String>,
}

impl<'de> Deserialize<'de> for LoggerConfig {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut config = Value::deserialize(deserializer)?;

        let log_level = config.pointer_and_deserialize::<D, _>(LOG_LEVEL_PTR)?;
        let service_name = config.pointer_and_deserialize::<D, _>(SERVICE_NAME_PTR)?;
        let traces_endpoint_ptr = config.pointer_mut(TRACES_ENDPOINT_PTR);

        let traces_endpoint = if let Some(ptr) = traces_endpoint_ptr {
            Some(from_value::<String>(ptr.take()).map_err(Error::custom)?)
        } else {
            None
        };

        Ok(LoggerConfig {
            log_level,
            service_name,
            traces_endpoint,
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

        let host = config.pointer_and_deserialize::<D, _>(HOST_PTR)?;
        let port = config.pointer_and_deserialize::<D, _>(PORT_PTR)?;
        let logger = LoggerConfig::deserialize(&config).map_err(Error::custom)?;
        let private = T::deserialize(&config).map_err(Error::custom)?;

        Ok(AppConfig::<T> {
            host,
            port,
            logger,
            private,
        })
    }
}

impl Default for AppConfig<Empty> {
    fn default() -> Self {
        AppConfig::builder()
            .add_default()
            .build()
            .expect("Default config never fails")
    }
}

impl<T: DeserializeOwned + Debug> AppConfig<T> {
    pub fn builder_with_private() -> AppConfigBuilder<T> {
        AppConfigBuilder::new()
    }

    pub fn default_with(file_path: &str, env_prefix: &str) -> Result<Self> {
        AppConfig::builder_with_private()
            .add_default()
            .add_file(file_path)
            .add_env_prefixed(env_prefix)
            .build()
    }
}

impl AppConfig<Empty> {
    pub fn builder() -> AppConfigBuilder<Empty> {
        AppConfigBuilder::new()
    }
}

#[derive(Debug, Default)]
pub struct AppConfigBuilder<T> {
    builder: ConfigBuilder<DefaultState>,
    phantom: PhantomData<T>,
}

impl<T: DeserializeOwned + Debug> AppConfigBuilder<T> {
    pub fn new() -> Self {
        Self {
            builder: ConfigBuilder::default(),
            phantom: PhantomData,
        }
    }

    pub fn build(self) -> Result<AppConfig<T>> {
        self.builder
            .build()?
            .try_deserialize_and_log::<AppConfig<T>>()
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
