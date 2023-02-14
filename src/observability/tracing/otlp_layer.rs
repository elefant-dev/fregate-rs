use crate::error::Result;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::{global, sdk, sdk::Resource, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use std::str::FromStr;
use tracing::Subscriber;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::reload::Handle;
use tracing_subscriber::{registry::LookupSpan, reload, EnvFilter, Layer};

/// Sets [`tracing_opentelemetry::layer()`] with given arguments.
/// Returns optional boxed [`Layer`] and optional [`Handle`] so filter might be changed in runtime.
/// Uses batched span processor with [`opentelemetry::runtime::Tokio`] runtime.
/// If traces_endpoint is [`None`] skips layer configuration and returns [`None`]
#[allow(clippy::type_complexity)]
pub fn otlp_layer<S>(
    trace_level: &str,
    component_name: &str,
    traces_endpoint: Option<&str>,
) -> Result<(
    Option<Box<dyn Layer<S> + Send + Sync>>,
    Option<Handle<EnvFilter, S>>,
)>
where
    S: Subscriber + for<'a> LookupSpan<'a> + Send + Sync,
{
    global::set_text_map_propagator(TraceContextPropagator::new());

    let trace_layer = if let Some(traces_endpoint) = traces_endpoint {
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(traces_endpoint),
            )
            .with_trace_config(sdk::trace::config().with_resource(Resource::new(vec![
                // Here it is service.name, but in our logs it is component_name.
                KeyValue::new("service.name", component_name.to_owned()),
            ])))
            .install_batch(opentelemetry::runtime::Tokio)?;

        let layer = tracing_opentelemetry::layer().with_tracer(tracer);

        let filter = EnvFilter::from_str(trace_level).unwrap_or_default();
        let (filter, reload) = reload::Layer::new(filter);

        let trace_layer = layer
            .with_filter(filter)
            .with_filter(filter_fn(|metadata| metadata.is_span()))
            .boxed();

        (Some(trace_layer), Some(reload))
    } else {
        (None, None)
    };

    Ok(trace_layer)
}
