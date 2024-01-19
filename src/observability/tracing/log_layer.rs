use crate::error::Result;
use crate::observability::tracing::{
    event_formatter::EventFormatter, COMPONENT, INSTANCE_ID, SERVICE, VERSION,
};
use std::str::FromStr;
use tracing::Subscriber;
use tracing_appender::non_blocking::{WorkerGuard, DEFAULT_BUFFERED_LINES_LIMIT};
use tracing_subscriber::{
    filter::EnvFilter, filter::Filtered, registry::LookupSpan, reload, reload::Handle, Layer,
};
use uuid;

/// Returns [`Layer`] with custom event formatter [`EventFormatter`]
/// Configured with non-blocking writer [`tracing_appender::non_blocking::NonBlocking`] to [`std::io::stdout()`]
#[allow(clippy::type_complexity)]
pub fn log_layer<S>(
    log_level: &str,
    version: &str,
    service_name: &str,
    component_name: &str,
    log_msg_length: Option<usize>,
    buffered_lines_limit: Option<usize>,
) -> Result<(
    Filtered<Box<dyn Layer<S> + Send + Sync>, reload::Layer<EnvFilter, S>, S>,
    Handle<EnvFilter, S>,
    WorkerGuard,
)>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let mut formatter = EventFormatter::new_with_limit(log_msg_length);

    formatter.add_default_field_to_events(VERSION, version)?;
    formatter.add_default_field_to_events(SERVICE, service_name)?;
    formatter.add_default_field_to_events(COMPONENT, component_name)?;
    formatter.add_default_field_to_events(INSTANCE_ID, uuid::Uuid::new_v4().to_string())?;

    let buffered_lines_limit = buffered_lines_limit.unwrap_or(DEFAULT_BUFFERED_LINES_LIMIT);

    let (writer, guard) = tracing_appender::non_blocking::NonBlockingBuilder::default()
        .lossy(true)
        .buffered_lines_limit(buffered_lines_limit)
        .finish(std::io::stdout());

    let layer = tracing_subscriber::fmt::layer()
        .with_writer(writer)
        .event_format(formatter)
        .boxed();

    let filter = EnvFilter::from_str(log_level).unwrap_or_default();
    let (filter, reload) = reload::Layer::new(filter);
    let layer = layer.with_filter(filter);

    Ok((layer, reload, guard))
}
