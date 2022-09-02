use crate::{DeserializeAndLog, Result};
use config::builder::DefaultState;
use config::{ConfigBuilder, Environment, File, FileFormat};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::net::IpAddr;

const DEFAULT_CONFIG: &str = include_str!("../resources/default_conf.toml");
const DEFAULT_SEPARATOR: &str = "_";

#[derive(Debug, Deserialize)]
pub struct Empty {}

#[derive(Debug, Deserialize)]
pub struct AppConfig<T> {
    #[serde(flatten)]
    pub private: T,
    pub host: IpAddr,
    pub port: u16,
}

impl Default for AppConfig<Empty> {
    fn default() -> Self {
        AppConfig::builder()
            .add_default()
            .build()
            .expect("Default config never fail")
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
