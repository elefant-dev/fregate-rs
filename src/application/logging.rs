//! Tools initialise logging and tracing
use crate::error::Result;
use crate::log_fmt::{fregate_layer, EventFormatter, COMPONENT, SERVICE, VERSION};
use once_cell::sync::OnceCell;
use opentelemetry::{global, sdk, sdk::Resource, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_zipkin::B3Encoding::MultipleHeader;
use std::str::FromStr;
use tracing::Subscriber;
use tracing_subscriber::filter::Filtered;
use tracing_subscriber::{
    filter::EnvFilter, layer::SubscriberExt, registry, registry::LookupSpan, reload,
    reload::Handle, util::SubscriberInitExt, Layer, Registry,
};

static HANDLE_LOG_LAYER: OnceCell<HandleLogLayer> = OnceCell::new();

/// Alias for [`Handle<Filtered<Box<dyn Layer<Registry> + Send + Sync>, EnvFilter, Registry>, Registry>`]
pub type HandleLogLayer =
    Handle<Filtered<Box<dyn Layer<Registry> + Send + Sync>, EnvFilter, Registry>, Registry>;

/// Return Some(&'static HandleLogLayer) if Handler is set up, otherwise return None
/// Initialised through [`init_tracing`] fn call
/// Used to change log level filter
pub fn get_handle_log_layer() -> Option<&'static HandleLogLayer> {
    HANDLE_LOG_LAYER.get()
}

/// Configures [`tracing_opentelemetry::OpenTelemetryLayer`] and returns [`Layer`]
pub fn get_trace_layer<S>(component_name: &str, traces_endpoint: &str) -> Result<impl Layer<S>>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    global::set_text_map_propagator(opentelemetry_zipkin::Propagator::with_encoding(
        MultipleHeader,
    ));

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
) -> Result<()> {
    let mut formatter = EventFormatter::new();

    formatter.add_default_field_to_events(VERSION, version)?;
    formatter.add_default_field_to_events(SERVICE, service_name)?;
    formatter.add_default_field_to_events(COMPONENT, component_name)?;

    let log_layer = fregate_layer(formatter).boxed();
    let filtered_log_layer =
        log_layer.with_filter(EnvFilter::from_str(log_level).unwrap_or_default());

    let (log_layer, reload_layer) = reload::Layer::new(filtered_log_layer);

    let trace_layer = traces_endpoint
        .map(|traces_endpoint| {
            let filtered_trace_layer = get_trace_layer(component_name, traces_endpoint)?
                .with_filter(EnvFilter::from_str(trace_level).unwrap_or_default());
            Result::Ok(filtered_trace_layer)
        })
        .transpose()?;

    registry().with(log_layer).with(trace_layer).try_init()?;

    let _ = HANDLE_LOG_LAYER.get_or_init(|| reload_layer);

    set_panic_hook();
    Ok(())
}
