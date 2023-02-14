//! [`HashBuilder`] struct wrapping [`std::collections::hash_map::RandomState`] or [`ahash::RandomState`] depends on feature "ahash"
#[cfg(feature = "ahash")]
use ahash::RandomState;
#[cfg(not(feature = "ahash"))]
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash, Hasher};

/// Alias for hash result type.
pub type HashId = u64;

/// Struct with sugar for easier hash calculation.
#[derive(Default, Debug)]
pub struct HashBuilder(RandomState);

impl HashBuilder {
    /// Creates new [`HashBuilder`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate hash for value.
    /// # Example
    /// ```rust
    /// use fregate::sugar::hash_builder::HashBuilder;
    ///
    /// let hash_builder = HashBuilder::new();
    ///
    /// let str0_hash = hash_builder.calculate_hash("str0");
    ///
    /// let num0 = u64::MAX;
    /// let num0_hash = hash_builder.calculate_hash(num0);
    /// let num0_ref_hash = hash_builder.calculate_hash(&num0);
    /// assert_eq!(num0_hash, num0_ref_hash);
    /// ```
    pub fn calculate_hash<T: Hash>(&self, value: T) -> HashId {
        let mut s = self.0.build_hasher();
        value.hash(&mut s);
        s.finish()
    }
}
