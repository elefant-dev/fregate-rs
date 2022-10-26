use crate::extensions::DeserializeExt;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::time::Duration;

const TLS_HANDSHAKE_TIMEOUT: &str = "/server/tls/handshake_timeout";
const TLS_KEY_PATH: &str = "/server/tls/key/path";
const TLS_CERTIFICATE_PATH: &str = "/server/tls/cert/path";

#[derive(Debug)]
pub struct TlsConfigurationVariables {
    /// TLS handshake timeout
    pub handshake_timeout: Duration,
    /// path to TLS key file
    pub key_path: Option<Box<str>>,
    /// path to TLS certificate file
    pub cert_path: Option<Box<str>>,
}

impl<'de> Deserialize<'de> for TlsConfigurationVariables {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let config = Value::deserialize(deserializer)?;

        let tls_handshake_timeout =
            config.pointer_and_deserialize::<u64, D::Error>(TLS_HANDSHAKE_TIMEOUT)?;
        let tls_key_path = config
            .pointer_and_deserialize::<_, D::Error>(TLS_KEY_PATH)
            .ok();
        let tls_cert_path = config
            .pointer_and_deserialize::<_, D::Error>(TLS_CERTIFICATE_PATH)
            .ok();

        Ok(Self {
            handshake_timeout: Duration::from_millis(tls_handshake_timeout),
            key_path: tls_key_path,
            cert_path: tls_cert_path,
        })
    }
}
