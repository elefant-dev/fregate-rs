use fregate::{AppConfigBuilder, DeserializeExt};
use serde::{
    de::{DeserializeOwned, Error as DeError},
    Deserialize, Deserializer,
};
use serde_json::Value;
use std::error::Error as StdError;

const SCALICA_ADDRESS_PATH: &str = "/scalica/address";
const SCALICA_NEW_ADDRESS_PATH: &str = "/scalica/new/address";

#[derive(Debug)]
struct Configuration<T: DeserializeOwned> {
    scalica_address: Box<str>,
    private: T,
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for Configuration<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let scalica_address = value.pointer_and_deserialize(SCALICA_ADDRESS_PATH)?;
        let private = T::deserialize(value).map_err(DeError::custom)?;

        Ok(Self {
            scalica_address,
            private,
        })
    }
}

#[derive(Debug)]
struct ExtendedConfiguration {
    scalica_new_address: Box<str>,
}

impl<'de> Deserialize<'de> for ExtendedConfiguration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let scalica_new_address = value.pointer_and_deserialize(SCALICA_NEW_ADDRESS_PATH)?;

        Ok(Self {
            scalica_new_address,
        })
    }
}

fn main() -> Result<(), Box<dyn StdError>> {
    std::env::set_var("BOHEMIA_SCALICA_ADDRESS", "Earth");
    std::env::set_var("BOHEMIA_SCALICA_NEW_ADDRESS", "Mars");

    let config = AppConfigBuilder::<Configuration<ExtendedConfiguration>>::new()
        .add_default()
        .add_env_prefixed("BOHEMIA")
        .build()?;

    assert_eq!(config.private.scalica_address.as_ref(), "Earth");
    assert_eq!(config.private.private.scalica_new_address.as_ref(), "Mars");

    println!("configuration: `{config:#?}`.");

    Ok(())
}
