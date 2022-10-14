#[allow(clippy::derive_partial_eq_without_eq, clippy::large_enum_variant)]
pub mod proto;

pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("./proto/description.bin");

pub const TLS_CERTIFICATE: &[u8] = include_bytes!("../certs/server.crt");
pub const TLS_PRIVATE_KEY: &[u8] = include_bytes!("../certs/server.key");
