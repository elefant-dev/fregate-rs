use crate::{DeserializeAndLog, Result};
use config::{Config, Environment, File};
use serde::Deserialize;
use std::fmt::Debug;
use std::net::IpAddr;

#[derive(Debug, Deserialize)]
pub struct DefaultConfig {
    pub server: DefaultServerConfig,
}

#[derive(Debug, Deserialize)]
pub struct DefaultServerConfig {
    pub host: IpAddr,
    pub port: u16,
}

// Do we want to deserialize it into provided struct, or always DefaultConfig ?
pub fn read_default_config(file_path: &str, env_prefix: &str) -> Result<DefaultConfig> {
    Config::builder()
        .add_source(File::with_name(file_path))
        .add_source(Environment::with_prefix(env_prefix).separator("_"))
        .build()?
        .try_deserialize_and_log::<DefaultConfig>()
}
