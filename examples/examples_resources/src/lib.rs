#[allow(clippy::derive_partial_eq_without_eq, clippy::large_enum_variant)]
pub mod proto;

pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("./proto/description.bin");
