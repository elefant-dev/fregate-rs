#[allow(clippy::derive_partial_eq_without_eq, clippy::large_enum_variant)]
pub mod proto;

pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("./proto/description.bin");

pub const TLS_CERT: &[u8] = include_bytes!("../certs/tls.cert");
pub const TLS_KEY: &[u8] = include_bytes!("../certs/tls.key");
