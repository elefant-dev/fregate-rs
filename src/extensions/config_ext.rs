use config::{Config, ConfigError};
use serde::Deserialize;
use std::fmt::Debug;
use tracing::{error, info};

// TODO(kos): Cleaning?
//TODO: This seems to be unused, remove in future when sure no need for config_ext
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Got config Error: `{0}`")]
    ConfigError(#[from] ConfigError),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait DeserializeAndLog {
    fn try_deserialize_and_log<'de, T>(self) -> Result<T>
    where
        T: Deserialize<'de> + Debug;
}

impl DeserializeAndLog for Config {
    fn try_deserialize_and_log<'de, T>(self) -> Result<T>
    where
        T: Deserialize<'de> + Debug,
    {
        let val = match T::deserialize(self) {
            Ok(config) => {
                info!("Configuration: `{config:?}`.", config = config);
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
