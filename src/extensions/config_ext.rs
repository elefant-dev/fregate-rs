use config::{Config, ConfigError};
use serde::Deserialize;
use std::fmt::Debug;
use tracing::{error, info};

// TODO(kos): Cleaning?
//TODO: This seems to be unused, remove in future when sure no need for config_ext
// TODO: Collect all Errors under 1 crate::Error
/// Possible Errors which migh occur in fregate
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Configuration Errors
    #[error("Got config Error: `{0}`")]
    ConfigError(#[from] ConfigError),
}

/// Result with [`Error`]
pub type Result<T> = std::result::Result<T, Error>;

// TODO(kos): Consider naming this trait as more idiomatic
//            `ConfigDeserializeAndLogExt` or just `ConfigExt`.
// TODO(kos): Consider sealing this trait with `#[sealed]`.
/// Used to deserialize and log error or deserialized value
pub trait DeserializeAndLog {
    /// deserialize and log self
    fn try_deserialize_and_log<'de, T>(self) -> Result<T>
    where
        T: Deserialize<'de> + Debug;
}

// TODO(kos): Potentially useless trait because purpose of the module is deafeated.
//
//            It's unclear whether this extension is worth to be in a library,
//            because it's quite natural in many cases for the logging system to
//            be initialized _after_ the config has been parsed, as the config
//            itself may contain options to configure the logging system.
impl DeserializeAndLog for Config {
    fn try_deserialize_and_log<'de, T>(self) -> Result<T>
    where
        T: Deserialize<'de> + Debug,
    {
        let val = match T::deserialize(self) {
            Ok(config) => {
                // TODO(kos): Starting from 1.58 Rust, the `config = config`
                //            part may be omitted, as done for the `err` in the
                //            log statement bellow.
                info!("Configuration: `{config:?}`.", config = config);
                Ok(config)
            }
            Err(err) => {
                // TODO(kos): Consider using more obvious and straightforward
                //            syntax `tracing::error!()` or `log::error!()`, as
                //            just an `error!()` may be confused with some
                //            third-party crates providing an `error!()` macro
                //            serving different purposes than logging.
                //            Just an `error!()` is too generic naming here,
                //            making a reader to recheck module imports to be
                //            sure he understood the semantics correctly.
                error!("Failed to deserialize config: {err}");
                Err(err.into())
            }
        };

        val
    }
}
