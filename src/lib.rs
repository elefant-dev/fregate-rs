#![deny(unused_must_use)]
#![warn(
    rust_2018_idioms,
    rust_2021_compatibility,
    missing_docs,
    missing_debug_implementations,
    clippy::expect_used,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn,
    clippy::panicking_unwrap,
    clippy::unwrap_used,
    clippy::if_let_mutex,
    clippy::map_unwrap_or,
    clippy::if_let_mutex,
    clippy::indexing_slicing,
    clippy::return_self_not_must_use
)]
#![warn(clippy::all)]
#![forbid(non_ascii_idents)]
#![forbid(unsafe_code)]

//! Developing an HTTP server requires to add code for logging, configuration, metrics, health checks etc.
//! This crate aims to solve these problems providing user with `Application` builder for setting up HTTP service.
//!
//! This project is in progress and might change a lot from version to version.
//!
//! # Examples
//!
//! Examples can be found [`here`](https://github.com/elefant-dev/fregate-rs/tree/main/examples).

mod application;

pub mod extensions;
pub mod middleware;
pub mod sugar;

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
