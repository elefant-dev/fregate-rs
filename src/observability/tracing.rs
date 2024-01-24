//! Tools initialise logging and tracing

mod event_formatter;
pub mod floor_char_boundary;
mod log_layer;
mod otlp_layer;
mod tracing_fields;

pub use event_formatter::*;
pub use log_layer::*;
pub use otlp_layer::*;
pub use tracing_fields::*;

use crate::error::Result;
use crate::observability::{HeadersFilter, HEADERS_FILTER};
use opentelemetry::global::set_error_handler;
use std::sync::OnceLock;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::Layered;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{
    filter::EnvFilter, layer::SubscriberExt, registry, reload::Handle, Layer, Registry,
};

/// Global value to be used everywhere.
pub const SANITIZED_VALUE: &str = "*****";

/// This by default uninitialised unless you call [`crate::bootstrap()`] or [`init_tracing`] functions.
/// Used to change log level filter
/// See in [`example`](https://github.com/elefant-dev/fregate-rs/tree/main/examples/log-level-change) how it might be used.
pub static LOG_LAYER_HANDLE: OnceLock<LogLayerHandle> = OnceLock::new();

/// This by default uninitialised unless you call [`crate::bootstrap()`] or [`init_tracing`] functions.
/// Used to change trace level filter
/// See in [`example`](https://github.com/elefant-dev/fregate-rs/tree/main/examples/log-level-change) how it might be used.
pub static OTLP_LAYER_HANDLE: OnceLock<TraceLayerHandle> = OnceLock::new();

/// Alias for [`Handle<EnvFilter, Layered<Option<Box<dyn Layer<Registry> + Send + Sync>>, Registry>>`]
pub type LogLayerHandle =
    Handle<EnvFilter, Layered<Option<Box<dyn Layer<Registry> + Send + Sync>>, Registry>>;

/// Alias for [`Handle<EnvFilter, Registry>`]
pub type TraceLayerHandle = Handle<EnvFilter, Registry>;

/// Set-up:\
/// 1. [`log_layer()`] with custom event formatter [`EventFormatter`].\
/// 2. [`otlp_layer()`].\
/// 3. Reload filters for both layers: [`OTLP_LAYER_HANDLE`] and [`LOG_LAYER_HANDLE`].\
/// 4. [`HEADERS_FILTER`] to be used in [`crate::extensions::HeaderFilterExt`].\
/// 5. Sets panic hook.\
/// Uses [`tracing_appender`] crate to do non blocking writes to stdout, so returns [`WorkerGuard`]. Read more here: [`https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/struct.WorkerGuard.html`]
#[allow(clippy::too_many_arguments)]
pub fn init_tracing(
    log_level: &str,
    trace_level: &str,
    version: &str,
    service_name: &str,
    component_name: &str,
    traces_endpoint: Option<&str>,
    log_msg_length: Option<usize>,
    buffered_lines_limit: Option<usize>,
    headers_filter: Option<HeadersFilter>,
    logging_path: Option<&str>,
    logging_file: Option<&str>,
) -> Result<WorkerGuard> {
    let (log_layer, log_reload, worker) = log_layer(
        log_level,
        version,
        service_name,
        component_name,
        log_msg_length,
        buffered_lines_limit,
        logging_path,
        logging_file,
    )?;
    let (otlp_layer, otlp_reload) = otlp_layer(trace_level, component_name, traces_endpoint)?;
    registry().with(otlp_layer).with(log_layer).try_init()?;

    let _ = LOG_LAYER_HANDLE.get_or_init(|| log_reload);
    if let Some(otlp_reload) = otlp_reload {
        let _ = OTLP_LAYER_HANDLE.get_or_init(|| otlp_reload);
    }
    if let Some(headers_filter) = headers_filter {
        let _ = HEADERS_FILTER.get_or_init(|| headers_filter);
    }

    set_error_handler(|err| {
        tracing::error!("{err}");
    })?;
    set_panic_hook();

    Ok(worker)
}

fn set_panic_hook() {
    // Capture the span context in which the program panicked
    std::panic::set_hook(Box::new(|panic| {
        // If the panic has a source location, record it as structured fields.
        if let Some(location) = panic.location() {
            tracing::error!(
                message = %panic,
                panic.file = location.file(),
                panic.line = location.line(),
                panic.column = location.column(),
            );
        } else {
            tracing::error!(message = %panic);
        }
    }));
}
