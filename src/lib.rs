#![warn(rust_2018_idioms, missing_debug_implementations, missing_docs)]
#![warn(clippy::all)]
#![forbid(non_ascii_idents)]
#![forbid(unsafe_code)]

//! Developing an HTTP server requires to add code for logging, configuration, metrics, health checks etc.
//! This crate aims to solve these problems providing user with `Application` builder for setting up HTTP service.
//!
//! This project is in progress and might change a lot from version to version.
//!
mod application;

pub mod extensions;
pub mod middleware;

#[doc(inline)]
pub use application::*;

pub use axum;
pub use config;
#[cfg(feature = "tls")]
pub use futures_util;
pub use hyper;
pub use thiserror;
pub use tokio;
pub use tonic;
pub use tower;
pub use tower_http;
pub use tracing;
pub use tracing_subscriber;
pub use valuable;
