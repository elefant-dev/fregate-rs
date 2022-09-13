use crate::AppConfig;
use opentelemetry::{global, sdk, sdk::trace::Tracer, sdk::Resource, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_zipkin::B3Encoding::MultipleHeader;
use std::str::FromStr;
use time::format_description::well_known::Rfc3339;
use tracing_opentelemetry::OpenTelemetryLayer as OTLayer;
use tracing_subscriber::{
    filter::EnvFilter,
    filter::Filtered,
    fmt,
    fmt::{
        format::{FmtSpan, Format, Json, JsonFields},
        layer,
        time::UtcTime,
    },
    layer::{Layered, SubscriberExt},
    registry, reload,
    reload::Handle,
    util::SubscriberInitExt,
    Layer, Registry,
};

type DefaultLayer = fmt::Layer<Registry, JsonFields, Format<Json, UtcTime<Rfc3339>>>;
type DefaultLayered = Layered<LogLayer, Registry>;

pub type LogFiltered = Filtered<DefaultLayer, EnvFilter, Registry>;
pub type LogLayer = reload::Layer<LogFiltered, Registry>;
pub type LogLayerReload = Handle<LogFiltered, Registry>;

pub type TraceFiltered = Filtered<OTLayer<DefaultLayered, Tracer>, EnvFilter, DefaultLayered>;
pub type TraceLayer = reload::Layer<TraceFiltered, DefaultLayered>;
pub type TraceLayerReload = Handle<TraceFiltered, DefaultLayered>;

fn get_log_filter<T>(config: &mut AppConfig<T>) -> LogLayer {
    let settings = &mut config.logger;

    let log_level = EnvFilter::from_str(settings.log_level.as_str()).unwrap_or_default();
    settings.log_level = log_level.to_string();

    let log_filter = layer()
        .json()
        .with_timer(UtcTime::rfc_3339())
        .flatten_event(true)
        .with_target(true)
        .with_current_span(false)
        .with_span_events(FmtSpan::NONE)
        .with_filter(log_level);

    let (log_filter, log_filter_reloader) = reload::Layer::new(log_filter);
    settings.log_filter_reloader.replace(log_filter_reloader);

    log_filter
}

fn get_trace_filter<T>(config: &mut AppConfig<T>) -> Option<TraceLayer> {
    let settings = &mut config.logger;

    if let Some(traces_endpoint) = &mut settings.traces_endpoint {
        global::set_text_map_propagator(opentelemetry_zipkin::Propagator::with_encoding(
            MultipleHeader,
        ));

        let service_name = settings.service_name.clone();
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(traces_endpoint.clone()),
            )
            .with_trace_config(sdk::trace::config().with_resource(Resource::new(vec![
                KeyValue::new("service.name", service_name),
            ])))
            // TODO: TOKIO RUNTIME
            .install_batch(opentelemetry::runtime::AsyncStd)
            .expect("failed to install opentelemetry_otlp pipeline");

        let trace_level = EnvFilter::from_str(settings.log_level.as_str()).unwrap_or_default();
        settings.trace_level = trace_level.to_string();

        let opentelemetry_layer = tracing_opentelemetry::layer()
            .with_tracer(tracer)
            .with_filter(trace_level);

        let (traces_filter, traces_filter_reloader) = reload::Layer::new(opentelemetry_layer);
        settings
            .traces_filter_reloader
            .replace(traces_filter_reloader);

        Some(traces_filter)
    } else {
        None
    }
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

pub(crate) fn init_tracing<T>(config: &mut AppConfig<T>) {
    let logs_filter = get_log_filter(config);
    let traces_filter = get_trace_filter(config);

    registry().with(logs_filter).with(traces_filter).init();
    set_panic_hook();
}
