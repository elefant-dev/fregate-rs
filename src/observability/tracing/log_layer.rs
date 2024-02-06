use crate::error::Result;
use crate::observability::tracing::writer::RollingFileWriter;
use crate::observability::tracing::{
    event_formatter::EventFormatter, COMPONENT, INSTANCE_ID, SERVICE, VERSION,
};
use crate::LoggerConfig;
use std::io::Write;
use std::str::FromStr;
use tracing::Subscriber;
use tracing_appender::non_blocking::{WorkerGuard, DEFAULT_BUFFERED_LINES_LIMIT};
use tracing_subscriber::{
    filter::EnvFilter, filter::Filtered, registry::LookupSpan, reload, reload::Handle, Layer,
};
use uuid;

/// Returns [`Layer`] with custom event formatter [`EventFormatter`]
/// Configured with non-blocking writer [`tracing_appender::non_blocking::NonBlocking`] to [`std::io::stdout()`]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn log_layer<S>(
    logger_config: &LoggerConfig,
    version: &str,
    service_name: &str,
    component_name: &str,
) -> Result<(
    Filtered<Box<dyn Layer<S> + Send + Sync>, reload::Layer<EnvFilter, S>, S>,
    Handle<EnvFilter, S>,
    WorkerGuard,
)>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let LoggerConfig {
        ref log_level,
        ref logging_path,
        ref logging_file,
        msg_length,
        buffered_lines_limit,
        logging_file_interval,
        logging_file_limit,
        logging_file_max_age,
        logging_file_max_count,
        logging_file_enable_zip: enable_zip,
        headers_filter: _,
    } = logger_config;

    let mut formatter = EventFormatter::new_with_limit(*msg_length);

    formatter.add_default_field_to_events(VERSION, version)?;
    formatter.add_default_field_to_events(SERVICE, service_name)?;
    formatter.add_default_field_to_events(COMPONENT, component_name)?;
    formatter.add_default_field_to_events(INSTANCE_ID, uuid::Uuid::new_v4().to_string())?;
    let buffered_lines_limit = buffered_lines_limit.unwrap_or(DEFAULT_BUFFERED_LINES_LIMIT);

    let dest: Box<dyn Write + Send + Sync + 'static> = if let Some(logging_path) = logging_path {
        let file_name_prefix = logging_file
            .as_deref()
            .map(|v| v.to_owned())
            .unwrap_or(format!("{component_name}.log"));

        let to_file = RollingFileWriter::new(
            logging_path,
            file_name_prefix,
            *logging_file_interval,
            *logging_file_limit,
            *logging_file_max_age,
            *logging_file_max_count,
            *enable_zip,
        );
        Box::new(to_file) as _
    } else {
        Box::new(std::io::stdout()) as _
    };

    let (writer, guard) = tracing_appender::non_blocking::NonBlockingBuilder::default()
        .lossy(true)
        .buffered_lines_limit(buffered_lines_limit)
        .finish(dest);

    let layer = tracing_subscriber::fmt::layer()
        .with_writer(writer)
        .event_format(formatter)
        .boxed();

    let filter = EnvFilter::from_str(log_level).unwrap_or_default();
    let (filter, reload) = reload::Layer::new(filter);
    let layer = layer.with_filter(filter);

    Ok((layer, reload, guard))
}
