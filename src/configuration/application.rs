use crate::configuration::observability::ObservabilityConfig;
use crate::configuration::source::ConfigSource;
use crate::{error::Result, extensions::DeserializeExt, ManagementConfig};
use config::{builder::DefaultState, ConfigBuilder, Environment, File, FileFormat};
use serde::{
    de::{DeserializeOwned, Error},
    Deserialize, Deserializer,
};
use serde_json::Value;
use std::marker::PhantomData;
use std::{fmt::Debug, net::IpAddr};
use tracing_appender::non_blocking::WorkerGuard;

#[cfg(feature = "tls")]
use crate::configuration::tls::TlsConfigurationVariables;

const HOST_PTR: &str = "/host";
const PORT_PTR: &str = "/port";
const PORT_SERVER_PTR: &str = "/server/port";
const MANAGEMENT_PTR: &str = "/management";

const DEFAULT_CONFIG: &str = include_str!("../resources/default_conf.toml");
const DEFAULT_SEPARATOR: &str = "_";

/// Default private config for [`AppConfig`].
#[derive(Deserialize, Debug, PartialEq, Eq, Copy, Clone)]
pub struct Empty {}

/// AppConfig reads and saves application configuration from different sources
#[derive(Debug)]
pub struct AppConfig<ConfigExt = Empty> {
    /// host address where to start Application
    pub host: IpAddr,
    /// When serialized uses `<PREFIX>`_PORT or `<PREFIX>`_SERVER_PORT names.
    /// `<PREFIX>`_SERVER_PORT has higher priority.
    pub port: u16,
    /// configuration for logs and traces
    pub observability_cfg: ObservabilityConfig,
    /// configures management endpoints
    pub management_cfg: ManagementConfig,
    /// TLS configuration parameters
    #[cfg(feature = "tls")]
    pub tls: TlsConfigurationVariables,
    /// field for each application specific configuration
    pub private: ConfigExt,
    /// Why it is here read more: [`https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/struct.WorkerGuard.html`]
    /// This one will not be cloned and will be set to [`None`] in clone.
    pub worker_guard: Option<WorkerGuard>,
}

impl<ConfigExt> Clone for AppConfig<ConfigExt>
where
    ConfigExt: Clone,
{
    fn clone(&self) -> Self {
        Self {
            host: self.host,
            port: self.port,
            observability_cfg: self.observability_cfg.clone(),
            management_cfg: self.management_cfg.clone(),
            #[cfg(feature = "tls")]
            tls: self.tls.clone(),
            private: self.private.clone(),
            worker_guard: None,
        }
    }
}

impl<'de, ConfigExt> Deserialize<'de> for AppConfig<ConfigExt>
where
    ConfigExt: Debug + DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let config = Value::deserialize(deserializer)?;
        let host = config.pointer_and_deserialize(HOST_PTR)?;
        let port = config
            .pointer_and_deserialize(PORT_SERVER_PTR)
            .or_else(|_err: D::Error| config.pointer_and_deserialize(PORT_PTR))?;

        let management_cfg = config
            .pointer_and_deserialize::<_, D::Error>(MANAGEMENT_PTR)
            .unwrap_or_default();
        let observability_cfg = ObservabilityConfig::deserialize(&config).map_err(Error::custom)?;
        #[cfg(feature = "tls")]
        let tls = TlsConfigurationVariables::deserialize(&config).map_err(Error::custom)?;
        let private = ConfigExt::deserialize(config).map_err(Error::custom)?;

        Ok(AppConfig::<ConfigExt> {
            host,
            port,
            observability_cfg,
            management_cfg,
            #[cfg(feature = "tls")]
            tls,
            private,
            worker_guard: None,
        })
    }
}

impl Default for AppConfig {
    #[allow(clippy::expect_used)]
    fn default() -> Self {
        AppConfig::builder()
            .add_default()
            .add_env_prefixed("OTEL")
            .build()
            .expect("Default config never fails")
    }
}

impl<ConfigExt> AppConfig<ConfigExt> {
    /// Creates [`AppConfigBuilder`] to add different sources to config
    pub fn builder() -> AppConfigBuilder<ConfigExt> {
        AppConfigBuilder::new()
    }

    /// Load file by given path and add environment variables with given prefix in addition to default config
    ///
    /// Environment variables have highet priority then file and then default configuration
    pub fn default_with(file_path: &str, env_prefix: &str) -> Result<Self>
    where
        ConfigExt: Debug + DeserializeOwned,
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
        ConfigExt: Debug + DeserializeOwned,
        S: IntoIterator<Item = ConfigSource<'a>>,
    {
        let mut config_builder = AppConfig::<ConfigExt>::builder()
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
pub struct AppConfigBuilder<ConfigExt> {
    builder: ConfigBuilder<DefaultState>,
    phantom: PhantomData<ConfigExt>,
}

impl<ConfigExt> AppConfigBuilder<ConfigExt> {
    /// Creates new [`AppConfigBuilder`]
    pub fn new() -> Self {
        Self {
            builder: ConfigBuilder::default(),
            phantom: PhantomData,
        }
    }

    /// Reads all registered sources
    pub fn build(self) -> Result<AppConfig<ConfigExt>>
    where
        ConfigExt: Debug + DeserializeOwned,
    {
        Ok(self
            .builder
            .build()?
            .try_deserialize::<AppConfig<ConfigExt>>()?)
    }

    /// Add default config
    #[must_use]
    pub fn add_default(mut self) -> Self {
        self.builder = self
            .builder
            .add_source(File::from_str(DEFAULT_CONFIG, FileFormat::Toml));
        self
    }

    /// Add file
    #[must_use]
    pub fn add_file(mut self, path: &str) -> Self {
        self.builder = self.builder.add_source(File::with_name(path));
        self
    }

    /// Add string
    #[must_use]
    pub fn add_str(mut self, str: &str, format: FileFormat) -> Self {
        self.builder = self.builder.add_source(File::from_str(str, format));
        self
    }

    /// Add environment variables with specified prefix and default separator: "_"
    #[must_use]
    pub fn add_env_prefixed(mut self, prefix: &str) -> Self {
        self.builder = self.builder.add_source(
            Environment::with_prefix(prefix)
                .try_parsing(true)
                .separator(DEFAULT_SEPARATOR),
        );
        self
    }
}
