use serde::de::Error;
use serde::Deserialize;
use serde_json::Value;

// TODO: FIXME: https://github.com/serde-rs/serde/issues/2261
// TODO(kos): Is that really necessary?
// Do we have `repr(transparent)`? Seems not.
// Why not just standard algorithm?
/// Needed for overcoming overlapping path in config deserialization.
pub trait DeserializeExt {
    fn pointer_and_deserialize<'de, T, E>(&'de self, pointer: &'static str) -> Result<T, E>
    where
        T: Deserialize<'de>,
        E: Error;
}

impl DeserializeExt for Value {
    fn pointer_and_deserialize<'de, T, E>(&'de self, pointer: &'static str) -> Result<T, E>
    where
        T: Deserialize<'de>,
        E: Error,
    {
        let raw_ret = self
            .pointer(pointer)
            .ok_or_else(|| E::missing_field(pointer))?;

        T::deserialize(raw_ret).map_err(E::custom)
    }
}
