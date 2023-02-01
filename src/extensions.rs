//! Extensions traits for different crates
mod axum_ext;
mod axum_tonic;
mod http_req_ext;
#[cfg(feature = "reqwest")]
mod reqwest_ext;
mod sanitize_ext;
mod serde_ext;
mod tonic_ext;

pub use axum_ext::*;
pub use axum_tonic::*;
pub use http_req_ext::*;
#[cfg(feature = "reqwest")]
pub use reqwest_ext::*;
pub use sanitize_ext::*;
pub use serde_ext::*;
pub use tonic_ext::*;
