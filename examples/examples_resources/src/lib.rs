pub mod grpc;
#[allow(clippy::derive_partial_eq_without_eq, clippy::large_enum_variant)]
pub mod proto;

use tonic::{
    body::BoxBody,
    codegen::http::{self},
    Status,
};

pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("./proto/description.bin");

pub async fn deny_middleware<B, Next>(
    _req: hyper::Request<B>,
    _next: Next,
) -> http::Response<BoxBody> {
    Status::permission_denied("You shall not pass").to_http()
}
