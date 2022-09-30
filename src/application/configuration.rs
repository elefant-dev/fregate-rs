use crate::DeserializeExt;
use config::{builder::DefaultState, ConfigBuilder, ConfigError, Environment, File, FileFormat};
use serde::{
    de::{DeserializeOwned, Error},
    Deserialize, Deserializer,
};
use serde_json::{from_value, Value};
use std::{fmt::Debug, marker::PhantomData, net::IpAddr};
use tracing::log::info;

// FIXME(kos): There is simpler way of loading config: just use Deserialize derive.
// Custom algorithm might has some advantages, but drawbacks are such
// - it makes extension of config difficult, every service has its own structur of config
// - it makes code more complicated
// After refactoring less than 50 lines will stay.
const HOST_PTR: &str = "/host";
const PORT_PTR: &str = "/port";
const LOG_LEVEL_PTR: &str = "/log/level";
const TRACE_LEVEL_PTR: &str = "/trace/level";
const SERVICE_NAME_PTR: &str = "/service/name";
const TRACES_ENDPOINT_PTR: &str = "/exporter/otlp/traces/endpoint";
const DEFAULT_CONFIG: &str = include_str!("../resources/default_conf.toml");
const DEFAULT_SEPARATOR: &str = "_";

#[derive(Clone, Debug)]
pub enum ConfigSource<'a> {
    String(&'a str, FileFormat),
    File(&'a str),
    EnvPrefix(&'a str),
}

#[derive(Deserialize, Debug, PartialEq, Eq, Copy, Clone)]
pub struct Empty {}

// FIXME(kos): ?
// https://serde.rs/field-attrs.html#flatten
//
#[derive(Debug, Deserialize)]
pub struct AppConfig<T> {
    pub host: IpAddr,
    pub port: u16,
    pub logger: LoggerConfig,
    pub private: T,
}

#[derive(Debug)]
pub struct LoggerConfig {
    pub log_level: String,
    pub trace_level: String,
    pub service_name: String,
    pub traces_endpoint: Option<String>,
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
            logger,
            private,
        })
    }
}

impl Default for AppConfig<Empty> {
    fn default() -> Self {
        AppConfig::builder()
            .add_default()
            .add_env_prefixed("OTEL")
            .build()
            .expect("Default config never fails")
    }
}

impl<T> AppConfig<T> {
    pub fn builder() -> AppConfigBuilder<T> {
        AppConfigBuilder::new()
    }

    pub fn default_with(file_path: &str, env_prefix: &str) -> Result<Self, ConfigError>
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

    pub fn load_from<'a, S>(sources: S) -> Result<Self, ConfigError>
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

// FIXME(kos): Why not exposing builder of crate `config`?
// Method add_default should be used every time.
// Methods add_file, add_str, add_env_prefixed are just wrappers and are barely useful.
#[derive(Debug, Default)]
pub struct AppConfigBuilder<T> {
    builder: ConfigBuilder<DefaultState>,
    phantom: PhantomData<T>,
}

// TODO(kos): Parameter `T` should be not near struct, but wher it's used, near method `build`.
impl<T> AppConfigBuilder<T> {
    pub fn new() -> Self {
        Self {
            builder: ConfigBuilder::default(),
            phantom: PhantomData,
        }
    }

    pub fn build(self) -> Result<AppConfig<T>, ConfigError>
    where
        T: Debug + DeserializeOwned,
    {
        let config = self.builder.build()?.try_deserialize::<AppConfig<T>>()?;

        info!("Configuration: `{config:?}`.");

        Ok(config)
    }

    pub fn add_default(mut self) -> Self {
        self.builder = self
            .builder
            .add_source(File::from_str(DEFAULT_CONFIG, FileFormat::Toml));
            // TODO: embedding of textual file into bin files has several disadvantages
            // 1. larger bin file
            // 2. slower initialization of program
            // 3. too late ( run-time ) information about broken toml file
            // Consider embedding all defaults into code.
        self
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
