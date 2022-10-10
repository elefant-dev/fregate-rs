use sealed::sealed;
use serde::de::Error;
use serde::Deserialize;
use serde_json::Value;

// TODO: FIXME: https://github.com/serde-rs/serde/issues/2261
// TODO(kos): Is that really necessary?
//            Do we have `repr(transparent)`? Seems not.
//            Consider to omit specifying JSON pointers explicitly and rely onto
//            `config` crate merging capabilities with possibly custom section
//            separators.
//            https://github.com/mehcode/config-rs/blob/0.13.2/examples/hierarchical-env/settings.rs
//            Thus, requiring some restructuring or providing some limitations
//            in edge cases, this scheme is much more simpler, straightforward,
//            and clear for high majority of cases.
/// Needed for overcoming overlapping path in config deserialization.
#[sealed]
pub trait DeserializeExt {
    /// find value by given pointer and try to deserialize
    fn pointer_and_deserialize<'de, T, E>(&'de self, pointer: &'static str) -> Result<T, E>
    where
        T: Deserialize<'de>,
        E: Error;
}

#[sealed]
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
