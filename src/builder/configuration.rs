use config::Config;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use tracing::{error, info};

const DEFAULT_PORT: u16 = 8000;
const DEFAULT_SEPARATOR: &str = "_";

#[derive(Debug, Default)]
pub struct ConfigurationBuilder<'a> {
    configuration_environment: Option<Environment<'a>>,
    configuration_file: Option<&'a str>,
}

#[derive(Debug)]
pub enum Environment<'a> {
    Simple,
    Prefixed(&'a str),
}

impl<'a> Default for Environment<'a> {
    fn default() -> Self {
        Environment::Simple
    }
}

impl<'a> ConfigurationBuilder<'a> {
    pub fn set_path_to_file(&mut self, file: &'a str) -> &mut Self {
        self.configuration_file = Some(file);
        self
    }

    pub fn set_environment(&mut self, environment: Environment<'a>) -> &mut Self {
        self.configuration_environment = Some(environment);
        self
    }

    pub fn build(&self) -> Config {
        let mut builder = Config::builder();

        if let Some(file) = self.configuration_file {
            builder = builder.add_source(config::File::with_name(file))
        }

        if let Some(env) = &self.configuration_environment {
            builder = builder.add_source(match env {
                Environment::Simple => config::Environment::default().separator(DEFAULT_SEPARATOR),
                Environment::Prefixed(prefix) => {
                    config::Environment::with_prefix(prefix).separator(DEFAULT_SEPARATOR)
                }
            })
        };

        let config = builder.build().expect("Failed to build config");

        info!(
            "Configuration: `{config}`.",
            config = config
                .clone()
                .try_deserialize::<serde_json::Value>()
                .expect("Failed to deserialize config")
        );

        config
    }
}

pub fn get_address(config: &Config) -> IpAddr {
    match config.get::<String>("server.host") {
        Ok(conf_address) => {
            if let Ok(ip4) = conf_address.parse::<Ipv4Addr>() {
                IpAddr::V4(ip4)
            } else if let Ok(ip6) = conf_address.parse::<Ipv6Addr>() {
                IpAddr::V6(ip6)
            } else {
                error!(
                    "Failed to parse address into Ipv4Addr and Ipv6Addr, going to start on {}",
                    Ipv4Addr::UNSPECIFIED
                );
                IpAddr::V4(Ipv4Addr::UNSPECIFIED)
            }
        }
        Err(_) => {
            error!(
                "Failed to read IpAddr from configuration, going to start on {}",
                Ipv4Addr::UNSPECIFIED
            );
            IpAddr::V4(Ipv4Addr::UNSPECIFIED)
        }
    }
}

pub fn get_port(config: &Config) -> u16 {
    match config.get::<u16>("server.port") {
        Ok(port) => port,
        Err(_) => {
            error!(
                "Failed to get port from configuration, going to start on {}",
                DEFAULT_PORT
            );
            DEFAULT_PORT
        }
    }
}
