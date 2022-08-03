use serde::Deserialize;
use std::fmt::Debug;
use std::net::IpAddr;

#[derive(Debug, Deserialize)]
pub struct DefaultConfig {
    pub server: DefaultServerConfig,
}

#[derive(Debug, Deserialize)]
pub struct DefaultServerConfig {
    #[serde(rename = "host")]
    pub ip_addr: IpAddr,
    pub port: u16,
}
