use crate::extensions::ConfigExt;
use crate::{error::Result, extensions::DeserializeExt};
use config::{builder::DefaultState, ConfigBuilder, File};
use serde::de::DeserializeOwned;
use serde::{de::Error, Deserialize, Deserializer};
use serde_json::{from_value, Value};
use std::{fmt::Debug, net::IpAddr};

const LOG_LEVEL_PTR: &str = "/log/level";
const TRACE_LEVEL_PTR: &str = "/trace/level";
const SERVICE_NAME_PTR: &str = "/service/name";
const TRACES_ENDPOINT_PTR: &str = "/exporter/otlp/traces/endpoint";

/// Configuration where to start server
#[derive(Debug, Clone, Deserialize)]
pub struct ApplicationConfig {
    /// host address where to start Application
    pub host: IpAddr,
    /// port
    pub port: u16,
}

/// configuration for logs and traces
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// log level read to string and later parsed into EnvFilter
    pub log_level: String,
    /// trace level read to string and later parsed into EnvFilter
    pub trace_level: String,
    /// service name to be used in opentelemetry exporter
    pub service_name: String,
    /// endpoint where to export traces
    pub traces_endpoint: Option<String>,
}

impl<'de> Deserialize<'de> for TracingConfig {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
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

        Ok(TracingConfig {
            log_level,
            trace_level,
            service_name,
            traces_endpoint,
        })
    }
}

/// Returns [`ConfigBuilder<DefaultState>`]
/// ```no_run
/// use fregate::config::{File, FileFormat};
/// use fregate::extensions::ConfigExt;
/// use fregate::init_tracing_from_config;
/// use fregate::tracing::info;
/// use fregate::{config_builder, TracingConfig, tokio, ApplicationConfig};
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug)]
/// pub struct CustomConfig {
///     pub number: u32,
///     #[serde(flatten)]
///     pub app_config: ApplicationConfig,
///     #[serde(flatten)]
///     pub tracing_config: TracingConfig,
/// }
///
/// #[tokio::main]
/// async fn main() {
///     std::env::set_var("TEST_PORT", "3333");
///     std::env::set_var("TEST_NUMBER", "1010");
///     std::env::set_var("TEST_LOG_LEVEL", "debug");
///     std::env::set_var("TEST_TRACE_LEVEL", "debug");
///     std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "http:///0.0.0.0:4317");
///     std::env::set_var("OTEL_SERVICE_NAME", "CONFIGURATION");
///
///     /// For more flexibility you may use builder
///     let custom_config: CustomConfig = config_builder()
///         .add_fregate_defaults()
///         .add_source(File::with_name("path_to_file"))
///         .add_env_prefixed("TEST")
///         .add_env_prefixed("OTEL")
///         .build()
///         .unwrap()
///         .try_deserialize()
///         .unwrap();
///
///     let tracing_config = custom_config.tracing_config.clone();
///
///     init_tracing_from_config(tracing_config).unwrap();
///     info!("Loaded {custom_config:?}");
///
/// }
/// ```
pub fn config_builder() -> ConfigBuilder<DefaultState> {
    ConfigBuilder::default()
}

/// Read default [`ApplicationConfig`].
///
/// This should never Fail
impl Default for ApplicationConfig {
    #[allow(clippy::expect_used)]
    fn default() -> Self {
        config_builder()
            .add_fregate_defaults()
            .build()
            .expect("Failed to build default config")
            .try_deserialize::<Self>()
            .expect("Failed to deserialize default config")
    }
}

/// Read default [`TracingConfig`].
///
/// This should never Fail
impl Default for TracingConfig {
    fn default() -> Self {
        config_builder()
            .add_fregate_defaults()
            .build()
            .expect("Failed to build default config")
            .try_deserialize::<Self>()
            .expect("Failed to deserialize default config")
    }
}

/// Load file by given path and add environment variables with given prefix in addition to default fregate config
///
/// Usage example:
/// ```
/// use fregate::config::{File, FileFormat};
/// use fregate::extensions::ConfigExt;
/// use fregate::init_tracing_from_config;
/// use fregate::tracing::info;
/// use fregate::{load_default_config_with, Application, TracingConfig};
/// use fregate::{tokio, ApplicationConfig};
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug)]
/// pub struct CustomConfig {
///     pub number: u32,
///     #[serde(flatten)]
///     pub app_config: ApplicationConfig,
///     #[serde(flatten)]
///     pub tracing_config: TracingConfig,
/// }
///
/// #[tokio::main]
/// async fn main() {
///     std::env::set_var("TEST_NUMBER", "1010");
///     std::env::set_var("TEST_PORT", "8005");
///
///     let custom_config: CustomConfig =
///         load_default_config_with(None, Some("TEST")).unwrap();
///
///     let tracing_config = custom_config.tracing_config.clone();
///     init_tracing_from_config(tracing_config).unwrap();
///     info!("Loaded {custom_config:?}");
/// }
///````
pub fn load_default_config_with<Config: DeserializeOwned + Debug>(
    file_path: Option<&str>,
    env_prefixed: Option<&str>,
) -> Result<Config> {
    let config = config_builder().add_fregate_defaults();

    let config = if let Some(file_path) = file_path {
        config.add_source(File::with_name(file_path))
    } else {
        config
    };

    let config = if let Some(env_prefixed) = env_prefixed {
        config.add_env_prefixed(env_prefixed)
    } else {
        config
    };

    let config = config.build()?.try_deserialize::<Config>()?;

    Ok(config)
}
