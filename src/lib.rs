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

//! Set of instruments to simplify http server set-up.\
//!
//! This project is in progress and might change a lot from version to version.
//!
//! Example:
//! ```no_run
//! use fregate::{
//!     axum::{routing::get, Router},
//!     bootstrap, tokio, AppConfig, Application,
//! };
//!
//! async fn handler() -> &'static str {
//!     "Hello, World!"
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let config: AppConfig = bootstrap([]).unwrap();
//!
//! Application::new(config)
//!         .router(Router::new().route("/", get(handler)))
//!         .serve()
//!         .await
//!         .unwrap();
//! }
//! ```
//!
//!
//! # Examples
//!
//! Examples can be found [`here`](https://github.com/elefant-dev/fregate-rs/tree/main/examples).

mod application;
mod static_assert;

pub mod bootstrap;
pub mod configuration;
pub mod error;
pub mod extensions;
pub mod middleware;
pub mod observability;
pub mod sugar;

#[doc(inline)]
pub use application::*;
#[doc(inline)]
pub use bootstrap::*;
#[doc(inline)]
pub use configuration::*;
#[allow(unused_imports)]
pub use static_assert::*;

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
