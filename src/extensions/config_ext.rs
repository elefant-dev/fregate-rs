use config::{Config, ConfigError};
use serde::Deserialize;
use std::fmt::Debug;
use tracing::{debug, error};

#[derive(thiserror::Error, Debug)]
pub enum DeserializeError {
    #[error("Got config Error: `{0}`")]
    ConfigError(#[from] ConfigError),
}

pub trait DeserializeAndLog {
    fn try_deserialize_and_log<'de, T>(self) -> Result<T, DeserializeError>
    where
        T: Deserialize<'de> + Debug;
}

impl DeserializeAndLog for Config {
    fn try_deserialize_and_log<'de, T>(self) -> Result<T, DeserializeError>
    where
        T: Deserialize<'de> + Debug,
    {
        let val = match T::deserialize(self) {
            Ok(config) => {
                debug!("Configuration: `{config:?}`.", config = config);
                Ok(config)
            }
            Err(err) => {
                error!("Failed to deserialize config: {err}");
                Err(err.into())
            }
        };

        val
    }
}
