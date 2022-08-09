use crate::{DeserializeAndLog, Result};
use config::builder::DefaultState;
use config::FileFormat::Toml;
use config::{ConfigBuilder, Environment, File};
use serde::Deserialize;
use std::fmt::Debug;
use std::net::IpAddr;

const DEFAULT_CONFIG: &str = include_str!("../resources/default_conf.toml");
const DEFAULT_SEPARATOR: &str = "_";

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig::builder()
            .add_default()
            .build()
            .expect("Default config never fail")
    }
}

impl AppConfig {
    pub fn builder() -> AppConfigBuilder {
        AppConfigBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct AppConfigBuilder {
    builder: ConfigBuilder<DefaultState>,
}

impl AppConfigBuilder {
    pub fn build(self) -> Result<AppConfig> {
        self.builder.build()?.try_deserialize_and_log::<AppConfig>()
    }

    pub fn add_default(mut self) -> Self {
        self.builder = self
            .builder
            .add_source(File::from_str(DEFAULT_CONFIG, Toml));
        self
    }

    pub fn add_file(mut self, path: &str) -> Self {
        self.builder = self.builder.add_source(File::with_name(path));
        self
    }

    pub fn add_env_prefixed(mut self, prefix: &str) -> Self {
        self.builder = self
            .builder
            .add_source(Environment::with_prefix(prefix).separator(DEFAULT_SEPARATOR));
        self
    }
}
