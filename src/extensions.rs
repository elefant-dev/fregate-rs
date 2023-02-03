//! Extensions traits for different crates
mod axum_ext;
mod axum_tonic;
mod headers_ext;
mod http_req_ext;
#[cfg(feature = "reqwest")]
mod reqwest_ext;
mod serde_ext;
mod tonic_ext;

pub use axum_ext::*;
pub use axum_tonic::*;
pub use headers_ext::*;
pub use http_req_ext::*;
#[cfg(feature = "reqwest")]
pub use reqwest_ext::*;
pub use serde_ext::*;
pub use tonic_ext::*;
