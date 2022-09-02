use serde::de::Error;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

pub trait DeserializeExt<'de0> {
    fn pointer_and_deserialize<'de1, D, R>(
        &'de1 self,
        pointer: &'static str,
    ) -> Result<R, D::Error>
    where
        D: Deserializer<'de0>,
        R: Deserialize<'de1>;
}

impl<'de0> DeserializeExt<'de0> for Value {
    fn pointer_and_deserialize<'de1, D, R>(&'de1 self, pointer: &'static str) -> Result<R, D::Error>
    where
        D: Deserializer<'de0>,
        R: Deserialize<'de1>,
    {
        let raw_ret = self
            .pointer(pointer)
            .ok_or_else(|| Error::missing_field(pointer))?;

        R::deserialize(raw_ret).map_err(Error::custom)
    }
}
