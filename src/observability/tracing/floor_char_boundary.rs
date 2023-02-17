//! Since Rust will panic on attempt to get slice on invalid char boundaries
//! It might be useful to find valid char boundaries and avoid looping over each element of string.

/// This is a copy of floor_char_boundary fn which is [`unstable`](https://github.com/rust-lang/rust/issues/93743).
/// Once it is stabilised this will be removed from fregate.
pub fn floor_char_boundary(val: &str, index: usize) -> usize {
    if index >= val.len() {
        val.len()
    } else {
        let lower_bound = index.saturating_sub(3);
        let new_index = val
            .as_bytes()
            .get(lower_bound..=index)
            .unwrap_or_default()
            .iter()
            .rposition(|b| is_utf8_char_boundary(*b));

        let new_index = match new_index {
            Some(val) => val,
            None => unreachable!("floor_char_boundary fn should never fail"),
        };

        lower_bound + new_index
    }
}

#[inline]
const fn is_utf8_char_boundary(byte: u8) -> bool {
    (byte as i8) >= -0x40
}
