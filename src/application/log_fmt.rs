//! Fregate [`FormatEvent`] trait implementation
use crate::error::{Error, Result};
use opentelemetry::trace::{SpanId, TraceContextExt};
use serde::{ser::SerializeMap, Serialize, Serializer};
use serde_json::Value;
use std::borrow::Cow;
use std::{collections::BTreeMap, fmt, mem, num::NonZeroU8};
use time::format_description::well_known::iso8601::{Config, Iso8601, TimePrecision};
use tracing::{field::Field, Event, Subscriber};
use tracing_appender::non_blocking::{WorkerGuard, DEFAULT_BUFFERED_LINES_LIMIT};
use tracing_opentelemetry::OtelData;
use tracing_subscriber::registry::{Extensions, SpanRef};
use tracing_subscriber::{
    fmt::{format, FmtContext, FormatEvent, FormatFields},
    registry::LookupSpan,
    Layer,
};

#[cfg(tracing_unstable)]
use crate::tracing_fields::TRACING_FIELDS_STRUCTURE_NAME;
#[cfg(tracing_unstable)]
use valuable_serde::Serializable;

pub(crate) const VERSION: &str = "version";
pub(crate) const SERVICE: &str = "service";
pub(crate) const COMPONENT: &str = "component";
pub(crate) const TARGET: &str = "target";
pub(crate) const MSG: &str = "msg";
pub(crate) const MESSAGE: &str = "message";
pub(crate) const LOG_LEVEL: &str = "LogLevel";
pub(crate) const TIME: &str = "time";
pub(crate) const TIMESTAMP: &str = "timestamp";
pub(crate) const TRACE_ID: &str = "traceId";
pub(crate) const SPAN_ID: &str = "spanId";

const DEFAULT_FIELDS: [&str; 11] = [
    VERSION, SERVICE, COMPONENT, TARGET, MSG, LOG_LEVEL, TIME, TIMESTAMP, MESSAGE, TRACE_ID,
    SPAN_ID,
];
const MIN_LOG_MESSAGE_LEN: usize = 256;

/// Returns [`tracing_subscriber::Layer`] with custom event formatter [`EventFormatter`]
pub fn fregate_layer<S>(
    formatter: EventFormatter,
    buffered_lines_limit: Option<usize>,
) -> (impl Layer<S>, WorkerGuard)
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let buffered_lines_limit = buffered_lines_limit.unwrap_or(DEFAULT_BUFFERED_LINES_LIMIT);

    let (writer, guard) = tracing_appender::non_blocking::NonBlockingBuilder::default()
        .lossy(false)
        .buffered_lines_limit(buffered_lines_limit)
        .finish(std::io::stdout());

    let layer = tracing_subscriber::fmt::layer()
        .with_writer(writer)
        .event_format(formatter);

    (layer, guard)
}

/// Fregate [`EventFormatter`]
///
/// Example:
/// ```
/// use fregate::log_fmt::{fregate_layer, EventFormatter};
/// use fregate::tokio;
/// use fregate::tracing::{debug, info, trace, warn};
/// use fregate::tracing_subscriber::{layer::SubscriberExt, registry, util::SubscriberInitExt};
///
/// #[tokio::main]
/// async fn main() {
///     let mut formatter = EventFormatter::new();
///     formatter.add_field_to_events("additional_field", "additional_value").unwrap();
///
///     let (log_layer, _guard) = fregate_layer(EventFormatter::new_with_limits(Some(1)), None);
///     registry().with(log_layer).init();
///
///     info!("info message");
///     debug!("debug message");
///     trace!("trace message");
///     warn!("warn message");
/// }
///```
///
///```json
/// {"additional_field":"additional_value","msg":"info message","target":"check_fregate","LogLevel":"INFO","time":1665672717498107000,"timestamp":"2022-10-13T14:51:57.498Z"}
/// {"additional_field":"additional_value","msg":"info message","target":"check_fregate","LogLevel":"DEBUG","time":1665672717498210000,"timestamp":"2022-10-13T14:51:57.498Z"}
/// {"additional_field":"additional_value","msg":"info message","target":"check_fregate","LogLevel":"TRACE","time":1665672717498247000,"timestamp":"2022-10-13T14:51:57.498Z"}
/// {"additional_field":"additional_value","msg":"info message","target":"check_fregate","LogLevel":"WARN","time":1665672717498279000,"timestamp":"2022-10-13T14:51:57.498Z"}
/// ```
#[derive(Debug, Clone, Default)]
pub struct EventFormatter {
    additional_fields: BTreeMap<String, Value>,
    msg_len: Option<usize>,
}

impl EventFormatter {
    /// This is equal to call [`EventFormatter::new_with_limits(None)`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates new [`EventFormatter`] with limits for msg field
    /// Example:
    /// ```rust
    ///  use fregate::log_fmt::{fregate_layer, EventFormatter};
    ///  use std::str::FromStr;
    ///  use tracing::info;
    ///  use tracing_subscriber::layer::SubscriberExt;
    ///  use tracing_subscriber::util::SubscriberInitExt;
    ///  use tracing_subscriber::{registry, EnvFilter, Layer};
    ///  
    ///  #[tokio::main]
    ///  async fn main() {
    ///     let (log_layer, _guard) = fregate_layer(EventFormatter::new_with_limits(Some(1)), None);
    ///  
    ///      registry()
    ///          .with(log_layer.with_filter(EnvFilter::from_str("info").unwrap()))
    ///          .init();
    ///  
    ///      info!("message");
    ///  }
    /// ```
    /// Will print next log message:
    /// ```json
    /// {"time":1673608228799729000,"timestamp":"2023-01-13T11:10:28.800Z","LogLevel":"INFO","target":"playground","msg":"m ..."}
    /// ```
    pub fn new_with_limits(msg_len: Option<usize>) -> Self {
        Self {
            additional_fields: Default::default(),
            msg_len,
        }
    }

    /// add key-value pair which will be added to all events\
    /// returns [`crate::error::Error`] if one of possible keys are added:
    /// ```rust
    /// pub(crate) const VERSION: &str = "version";
    /// pub(crate) const SERVICE: &str = "service";
    /// pub(crate) const COMPONENT: &str = "component";
    /// pub(crate) const TARGET: &str = "target";
    /// pub(crate) const MSG: &str = "msg";
    /// pub(crate) const MESSAGE: &str = "message";
    /// pub(crate) const LOG_LEVEL: &str = "LogLevel";
    /// pub(crate) const TIME: &str = "time";
    /// pub(crate) const TIMESTAMP: &str = "timestamp";
    /// pub(crate) const TRACE_ID: &str = "traceId";
    /// pub(crate) const SPAN_ID: &str = "spanId";
    /// ```
    pub fn add_field_to_events<V: Serialize>(&mut self, key: &str, value: V) -> Result<()> {
        if DEFAULT_FIELDS.contains(&key) {
            Err(Error::CustomError(format!(
                "Prohibited to add key: '{key}' to EventFormatter"
            )))
        } else {
            self.add_default_field_to_events(key, value)
        }
    }

    pub(crate) fn add_default_field_to_events<V: Serialize>(
        &mut self,
        key: &str,
        value: V,
    ) -> Result<()> {
        let val = serde_json::to_value(value)?;
        self.additional_fields.insert(key.to_owned(), val);
        Ok(())
    }
}

impl<S, N> FormatEvent<S, N> for EventFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let serialize = || {
            let mut buf = Vec::with_capacity(MIN_LOG_MESSAGE_LEN);
            let mut serializer = serde_json::Serializer::new(&mut buf);
            let mut map_fmt = serializer.serialize_map(None)?;

            let mut visitor = JsonVisitor::new();
            event.record(&mut visitor);
            let mut event_storage = visitor.storage;

            let message = event_storage
                .remove(MESSAGE)
                .map(|mut msg| {
                    if let Some(limit) = self.msg_len {
                        limit_str_value(&mut msg, limit);
                    }
                    msg
                })
                .unwrap_or_default();
            let mut event_fields = event_storage.iter().filter(|(key, _)| {
                !DEFAULT_FIELDS.contains(&key.as_ref())
                    && !self.additional_fields.contains_key(key.as_ref())
            });
            let mut additional_fields = self.additional_fields.iter();
            let target = event.metadata().target();
            let level = event.metadata().level();
            let time = time::OffsetDateTime::now_utc();
            let time_ns = time.unix_timestamp_nanos();
            let timestamp = time.format(
                &Iso8601::<
                    {
                        Config::DEFAULT
                            .set_time_precision(TimePrecision::Second {
                                decimal_digits: NonZeroU8::new(3),
                            })
                            .encode()
                    },
                >,
            );
            let tracing_fields = ctx
                .lookup_current()
                .as_ref()
                .map(SpanRef::extensions)
                .as_ref()
                .and_then(Extensions::get::<OtelData>)
                .and_then(|otel_data| {
                    if otel_data.parent_cx.has_active_span() {
                        Some(otel_data.parent_cx.span().span_context().trace_id())
                    } else {
                        otel_data.builder.trace_id
                    }
                    .map(|trace_id| {
                        let span_id = otel_data.builder.span_id.unwrap_or(SpanId::INVALID);
                        (span_id, trace_id)
                    })
                });

            // serialize time
            map_fmt.serialize_entry(TIME, &time_ns)?;
            if let Ok(timestamp) = timestamp {
                map_fmt.serialize_entry(TIMESTAMP, timestamp.as_str())?;
            }

            // serialize event metadata
            map_fmt.serialize_entry(LOG_LEVEL, level.as_str())?;
            map_fmt.serialize_entry(TARGET, target)?;

            // If event under span serialize traceId and spanId
            if let Some((span_id, trace_id)) = tracing_fields {
                map_fmt.serialize_entry(TRACE_ID, &trace_id.to_string())?;
                map_fmt.serialize_entry(SPAN_ID, &span_id.to_string())?;
            }

            // serialize additional fields
            additional_fields.try_for_each(|(k, v)| map_fmt.serialize_entry(k, v))?;

            // Limit msg field
            map_fmt.serialize_entry(MSG, &message)?;

            // serialize event fields
            event_fields.try_for_each(|(k, v)| map_fmt.serialize_entry(k, v))?;

            map_fmt.end()?;
            Ok(buf)
        };

        let buffer: std::result::Result<Vec<u8>, std::io::Error> = serialize();

        match buffer {
            Ok(formatted) => match std::str::from_utf8(&formatted) {
                Ok(str) => {
                    write!(writer, "{str}")?;
                }
                Err(_) => {
                    write!(writer, "{}", String::from_utf8_lossy(&formatted))?;
                }
            },
            Err(err) => {
                write!(writer, "{err}")?;
            }
        }

        writeln!(writer)
    }
}

#[derive(Clone, Debug, Default)]
struct JsonVisitor<'a> {
    storage: BTreeMap<Cow<'a, str>, Value>,
}

impl<'a> JsonVisitor<'a> {
    fn new() -> Self {
        Self {
            storage: Default::default(),
        }
    }

    fn insert_owned<T: Serialize>(&mut self, key: String, value: T) {
        let value = serde_json::json!(value);
        self.storage.insert(Cow::Owned(key), value);
    }

    fn insert_borrowed<T: Serialize>(&mut self, key: &'a str, value: T) {
        let value = serde_json::json!(value);
        self.storage.insert(Cow::Borrowed(key), value);
    }
}

impl<'a> tracing::field::Visit for JsonVisitor<'a> {
    #[cfg(tracing_unstable)]
    fn record_value(&mut self, field: &Field, value: valuable::Value<'_>) {
        let mut serde_value = serde_json::json!(Serializable::new(value));
        let structure = value.as_structable();

        if let Some(structure) = structure {
            let definition = structure.definition();

            if definition.name() == TRACING_FIELDS_STRUCTURE_NAME {
                match serde_value.as_object_mut() {
                    Some(value) => {
                        let value = mem::take(value);
                        value.into_iter().for_each(|(k, v)| {
                            self.insert_owned(k, v);
                        });
                        return;
                    }
                    None => {
                        unreachable!("Named structure should always be mapped to serde_json::Value::Object()")
                    }
                }
            }
        }

        self.insert_borrowed(field.name(), serde_value)
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.insert_borrowed(field.name(), value);
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.insert_borrowed(field.name(), value);
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.insert_borrowed(field.name(), value);
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.insert_borrowed(field.name(), value);
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.insert_borrowed(field.name(), value);
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.insert_borrowed(field.name(), value.to_string());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        match field.name() {
            name if name.starts_with("r#") => {
                self.insert_borrowed(&name[2..], format!("{value:?}"));
            }
            name => {
                self.insert_borrowed(name, format!("{value:?}"));
            }
        };
    }
}

// TODO: remove when done
// https://github.com/rust-lang/rust/issues/93743
mod round {
    pub(crate) fn floor_char_boundary(val: &str, index: usize) -> usize {
        if index >= val.len() {
            val.len()
        } else {
            let lower_bound = index.saturating_sub(3);
            let new_index = val.as_bytes()[lower_bound..=index]
                .iter()
                .rposition(|b| is_utf8_char_boundary(*b));

            let new_index = match new_index {
                Some(val) => val,
                None => unreachable!("floor_char_boundary fn should never fail"),
            };

            lower_bound + new_index
        }
    }

    #[inline]
    const fn is_utf8_char_boundary(byte: u8) -> bool {
        (byte as i8) >= -0x40
    }
}

fn limit_str_value(value: &mut Value, limit: usize) {
    if let Value::String(str) = value {
        if str.len() > limit {
            let new_limit = round::floor_char_boundary(str, limit);
            str.replace_range(new_limit..str.len(), " ...");
        }
    }
}
