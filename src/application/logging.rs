//! Tools initialise logging and tracing
use crate::error::Result;
use crate::headers::{HeadersFilter, HEADERS_FILTER};
use crate::log_fmt::{fregate_layer, EventFormatter, COMPONENT, SERVICE, VERSION};
use once_cell::sync::OnceCell;
use opentelemetry::global::set_error_handler;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::{global, sdk, sdk::Resource, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use std::str::FromStr;
use tracing::Subscriber;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::Layered;
use tracing_subscriber::{
    filter::{filter_fn, EnvFilter},
    layer::SubscriberExt,
    registry,
    registry::LookupSpan,
    reload,
    reload::Handle,
    util::SubscriberInitExt,
    Layer, Registry,
};

/// Global value to be used everywhere.
pub const SANITIZED_VALUE: &str = "*****";

/// This by default uninitialised unless you call [`crate::bootstrap`] or [`init_tracing`] functions.
/// Used to change log level filter
pub static LOG_LAYER_HANDLE: OnceCell<LogLayerHandle> = OnceCell::new();

/// This by default uninitialised unless you call [`crate::bootstrap`] or [`init_tracing`] functions.
/// Used to change trace level filter
pub static TRACE_LAYER_HANDLE: OnceCell<TraceLayerHandle> = OnceCell::new();

/// Alias for [`Handle<EnvFilter, Layered<Option<Box<dyn Layer<Registry> + Send + Sync>>, Registry>>`]
pub type LogLayerHandle =
    Handle<EnvFilter, Layered<Option<Box<dyn Layer<Registry> + Send + Sync>>, Registry>>;

/// Alias for [`Handle<EnvFilter, Registry>`]
pub type TraceLayerHandle = Handle<EnvFilter, Registry>;

/// Configures [`tracing_opentelemetry::OpenTelemetryLayer`] and returns [`Layer`]
pub fn get_trace_layer<S>(component_name: &str, traces_endpoint: &str) -> Result<impl Layer<S>>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(traces_endpoint),
        )
        .with_trace_config(sdk::trace::config().with_resource(Resource::new(vec![
            // TODO: Here it is service.name, but we will have component.name
            KeyValue::new("service.name", component_name.to_owned()),
        ])))
        .install_batch(opentelemetry::runtime::Tokio)?;

    let trace_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    Ok(trace_layer)
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

/// Sets up:\
/// 1. [`fregate_layer`] with custom event formatter [`EventFormatter`].\
/// 2. [`tracing_opentelemetry::layer()`].\
/// 3. Reload filters for both layers: [`TRACE_LAYER_HANDLE`] and [`LOG_LAYER_HANDLE`].\
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
) -> Result<WorkerGuard> {
    let mut formatter = EventFormatter::new_with_limits(log_msg_length);

    formatter.add_default_field_to_events(VERSION, version)?;
    formatter.add_default_field_to_events(SERVICE, service_name)?;
    formatter.add_default_field_to_events(COMPONENT, component_name)?;

    let (log_layer, guard) = fregate_layer(formatter, buffered_lines_limit);
    let log_filter = EnvFilter::from_str(log_level).unwrap_or_default();
    let (log_filter, reload_log_filter) = reload::Layer::new(log_filter);
    let log_layer = log_layer.with_filter(log_filter);

    let trace_layer = if let Some(traces_endpoint) = traces_endpoint {
        let trace_filter = EnvFilter::from_str(trace_level).unwrap_or_default();
        let (filter, reload_trace_filter) = reload::Layer::new(trace_filter);

        let trace_layer = get_trace_layer(component_name, traces_endpoint)?;
        let trace_layer = trace_layer
            .with_filter(filter)
            .with_filter(filter_fn(|metadata| metadata.is_span()))
            .boxed();

        let _ = TRACE_LAYER_HANDLE.get_or_init(|| reload_trace_filter);
        Some(trace_layer)
    } else {
        None
    };

    if let Some(headers_filter) = headers_filter {
        HEADERS_FILTER.get_or_init(|| headers_filter);
    }

    registry().with(trace_layer).with(log_layer).try_init()?;
    let _ = LOG_LAYER_HANDLE.get_or_init(|| reload_log_filter);

    set_error_handler(|err| {
        tracing::error!("{err}");
    })?;
    set_panic_hook();

    Ok(guard)
}
