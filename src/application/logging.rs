//! Tools initialise logging and tracing
use crate::error::Result;
use crate::log_fmt::{fregate_layer, EventFormatter, COMPONENT, SERVICE, VERSION};
use once_cell::sync::OnceCell;
use opentelemetry::global::set_error_handler;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::{global, sdk, sdk::Resource, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use std::str::FromStr;
use tracing::Subscriber;
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

static HANDLE_LOG_LAYER: OnceCell<HandleLogLayer> = OnceCell::new();
static HANDLE_TRACE_LAYER: OnceCell<HandleTraceLayer> = OnceCell::new();

/// Alias for [`Handle<EnvFilter, Layered<Option<Box<dyn Layer<Registry> + Send + Sync>>, Registry>>`]
pub type HandleLogLayer =
    Handle<EnvFilter, Layered<Option<Box<dyn Layer<Registry> + Send + Sync>>, Registry>>;

/// Alias for [`Handle<EnvFilter, Registry>`]
pub type HandleTraceLayer = Handle<EnvFilter, Registry>;

/// Return Some(&'static HandleLogLayer) if Handler is set up, otherwise return None
/// Initialised through [`init_tracing`] fn call
/// Used to change log level filter
pub fn get_log_filter() -> Option<&'static HandleLogLayer> {
    HANDLE_LOG_LAYER.get()
}

/// Return [`Some(&'static Handle<EnvFilter, Registry>)`] if Handler is set up, otherwise return None
/// Initialised through [`init_tracing`] fn call
/// Used to change trace level filter
pub fn get_trace_filter() -> Option<&'static HandleTraceLayer> {
    HANDLE_TRACE_LAYER.get()
}

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

/// Set up global subscriber with formatting log layer to print logs in json format to console and if traces_endpoint is provided opentelemetry exporter to send traces to grafana
pub fn init_tracing(
    log_level: &str,
    trace_level: &str,
    version: &str,
    service_name: &str,
    component_name: &str,
    traces_endpoint: Option<&str>,
    log_msg_length: Option<usize>,
) -> Result<()> {
    let mut formatter = EventFormatter::new_with_limits(log_msg_length);

    formatter.add_default_field_to_events(VERSION, version)?;
    formatter.add_default_field_to_events(SERVICE, service_name)?;
    formatter.add_default_field_to_events(COMPONENT, component_name)?;

    let log_layer = fregate_layer(formatter);
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

        let _ = HANDLE_TRACE_LAYER.get_or_init(|| reload_trace_filter);
        Some(trace_layer)
    } else {
        None
    };

    registry().with(trace_layer).with(log_layer).try_init()?;
    let _ = HANDLE_LOG_LAYER.get_or_init(|| reload_log_filter);

    set_error_handler(|err| {
        tracing::error!("{err}");
    })?;
    set_panic_hook();

    Ok(())
}
