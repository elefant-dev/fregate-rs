//! Tools initialise logging and tracing
use crate::error::Result;
use crate::log_fmt::{fregate_layer, HandleFregateLogLayer};
use once_cell::sync::OnceCell;
use opentelemetry::{global, sdk, sdk::Resource, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_zipkin::B3Encoding::MultipleHeader;
use std::str::FromStr;
use tracing::Subscriber;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{
    filter::EnvFilter, layer::SubscriberExt, registry, reload, util::SubscriberInitExt, Layer,
};

static HANDLE_LOG_LAYER: OnceCell<HandleFregateLogLayer> = OnceCell::new();

/// Return Some(&'static HandleLogLayer) if Handler is set up, otherwise return None
///
/// Used to change log level filter
pub fn get_handle_log_layer() -> Option<&'static HandleFregateLogLayer> {
    HANDLE_LOG_LAYER.get()
}

// TODO: bug ? trace_id is not generated when used with reload Layer
// let (traces_filter, traces_filter_reloader) = reload::Layer::new(opentelemetry_layer);
// settings
//     .traces_filter_reloader
//     .replace(traces_filter_reloader);
fn get_trace_layer<S>(
    trace_level: &str,
    component_name: &str,
    traces_endpoint: &str,
) -> Result<impl Layer<S>>
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

    let trace_level = EnvFilter::from_str(trace_level).unwrap_or_default();

    let trace_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_filter(trace_level);

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
    let log_layer = fregate_layer(version, service_name, component_name, log_level)?;
    let (log_layer, reload_layer) = reload::Layer::new(log_layer);

    let trace_layer = if let Some(traces_endpoint) = traces_endpoint {
        Some(get_trace_layer(
            trace_level,
            component_name,
            traces_endpoint,
        )?)
    } else {
        None
    };

    registry().with(log_layer).with(trace_layer).try_init()?;
    let _ = HANDLE_LOG_LAYER.get_or_init(|| reload_layer);

    set_panic_hook();
    Ok(())
}
