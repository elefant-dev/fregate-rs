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

// TODO(kos): Use `#[doc(inline)]` whenever you reexport something publicly to have excellent documentation.

// #[doc(inline)]
pub use application::*;
// #[doc(inline)]
pub use extensions::*;
// #[doc(inline)]
pub use middleware::*;

// TODO(kos):
// Crate root namespace should be reserved for entities that will be immediately neede by the user.
// Currently everything is exported through the crate root.
// What is somewhat excessive.
// Structs like NoHealth and AlwaysReadyAndAlive should not pollute the root namespace.
// https://www.lurklurk.org/effective-rust/wildcard.html

pub use axum;
pub use config;
pub use hyper;
pub use thiserror;
pub use tonic;
pub use tower;
pub use tower_http;
pub use tracing;
