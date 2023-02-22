//! See in [`examples`](https://github.com/elefant-dev/fregate-rs/blob/main/examples/configuration/src/main.rs) how to configure your [`crate::Application`]
mod application;
mod observability;
mod source;

mod management;
#[cfg(feature = "tls")]
mod tls;

#[doc(inline)]
pub use application::*;
#[doc(inline)]
pub use management::*;
#[doc(inline)]
pub use observability::*;
#[doc(inline)]
pub use source::*;
