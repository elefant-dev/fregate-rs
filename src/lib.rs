//TODO: add docs and missing_docs lint
//TODO: clippy::expect_used,
#![warn(
    rust_2018_idioms,
    missing_debug_implementations,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn,
    clippy::panicking_unwrap,
    clippy::unwrap_used
)]

mod application;
mod extensions;
mod middleware;

pub use application::*;
pub use extensions::*;
pub use middleware::*;

pub use axum;
pub use config;
pub use hyper;
pub use thiserror;
pub use tonic;
pub use tower;
pub use tower_http;
pub use tracing;
