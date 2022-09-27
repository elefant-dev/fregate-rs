#![warn(
    rust_2018_idioms,
    missing_debug_implementations,
    missing_docs,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn,
    clippy::expect_used,
    clippy::panicking_unwrap,
    clippy::unwrap_used,
    clippy::inefficient_to_string,
    clippy::if_let_mutex
)]

//! Developing an HTTP server requires to add code for logging, configuration, metrics, health checks etc.
//! This crate aims to solve these problems providing user with `Application` builder for setting up HTTP service.
//!
//! This project is in progress and might change a lot from version to version.
//!
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
