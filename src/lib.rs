//TODO: add docs and missing_docs lint
//TODO: clippy::expect_used,

// TODO(kos): If your library is not intended to have `unsafe`, consider adding
//            `#![forbid(unsafe_code)]`, which will ease reasoning about the
//            code for potential security auditors, and will require quite a
//            reasoning to add any `unsafe` in future.
// TODO(kos): If your library is not intended to use any exotic notations and
//            idents, consider using `#![forbid(non_ascii_idents)]`, which will
//            prevent possible fuckups with naming and unicode misspelling.
// TODO(kos): Consider adding the whole `clippy::all`. Having more restriction
//            in libraries is good, as gives stricter rules for all the
//            contributors. Especially if it's open source.

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

// TODO(kos): Use `#[doc(inline)]` whenever you reexport something publicly to have excellent documentation.

// #[doc(inline)]
pub use application::*;
// #[doc(inline)]
pub use extensions::*;
// #[doc(inline)]
pub use middleware::*;

// TODO(kos): Crate root namespace should be reserved for entities that will be
//            immediately needed by the user.
//            Currently, everything is exported through the crate root.
//            What is somewhat excessive.
//            Structs like `NoHealth` and `AlwaysReadyAndAlive` should not
//            pollute the root namespace.
//            https://www.lurklurk.org/effective-rust/wildcard.html
//            Consider using more Rust-style paths, when referring to types.
//            Instead:
//            ```rust
//            use fregate::{NoHealth, AlwaysReadyAndAlive};
//            ```
//            just:
//            ```rust
//            use fregate::health;
//            let healthz = health::No;
//            let healthz = health::AlwaysReadyAndAlive;
//            ```

pub use axum;
pub use config;
pub use hyper;
pub use thiserror;
pub use tonic;
pub use tower;
pub use tower_http;
pub use tracing;
