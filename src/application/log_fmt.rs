//! Fregate [`FormatEvent`] trait implementation
use crate::error::Result;
use opentelemetry::trace::{SpanId, TraceContextExt};
use serde::{ser::SerializeMap, Serialize, Serializer};
use serde_json::Value;
use std::{collections::BTreeMap, fmt, mem, num::NonZeroU8};
use time::format_description::well_known::iso8601::{Config, Iso8601, TimePrecision};
use tracing::{field::Field, Event, Subscriber};
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
const MAX_MESSAGE_LEN: usize = 16 * 1024;
const AVERAGE_MESSAGE_LEN: usize = 1024;
const DEFAULT_FIELDS_LEN: usize = 256;

/// Returns [`tracing_subscriber::Layer`] with custom event formatter [`EventFormatter`]
pub fn fregate_layer<S>(formatter: EventFormatter) -> impl Layer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    tracing_subscriber::fmt::layer().event_format(formatter)
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
///     let mut formatter = EventFormatter::default();
///     formatter.add_field_to_events("additional_field", "additional_value").unwrap();
///     registry().with(fregate_layer(formatter)).init();
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
#[derive(Debug)]
pub struct EventFormatter {
    additional_fields: BTreeMap<String, Value>,
    max_message_len: usize,
    average_message_len: usize,
    fields_len: usize,
}

impl Default for EventFormatter {
    fn default() -> Self {
        Self {
            additional_fields: Default::default(),
            max_message_len: MAX_MESSAGE_LEN,
            average_message_len: AVERAGE_MESSAGE_LEN,
            fields_len: 0,
        }
    }
}

impl EventFormatter {
    /// Creates new [`EventFormatter`]
    pub fn new(max_message_size: usize) -> Self {
        assert!(
            max_message_size >= DEFAULT_FIELDS_LEN,
            "Max message size should be >= 256."
        );

        Self {
            additional_fields: Default::default(),
            max_message_len: max_message_size,
            average_message_len: AVERAGE_MESSAGE_LEN,
            fields_len: 0,
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
    pub fn add_field_to_events(&mut self, key: &str, value: &str) -> Result<()> {
        if DEFAULT_FIELDS.contains(&key) {
            Err(crate::error::Error::CustomError(format!(
                "Prohibited to add key: {key} to EventFormatter"
            )))
        } else {
            self.add_default_field_to_events(key, value)
        }
    }

    pub(crate) fn add_default_field_to_events(&mut self, key: &str, value: &str) -> Result<()> {
        // this is for each kv pair symbols in resulting message: "":"",
        const KV_SYMBOLS_SIZE: usize = 6;

        let val = serde_json::to_value(value)?;
        self.additional_fields.insert(key.to_owned(), val);
        self.fields_len += key.len() + value.len() + KV_SYMBOLS_SIZE;
        Ok(())
    }

    fn fields_len(&self) -> usize {
        self.additional_fields.len()
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
            let event_fields_len = event.fields().count();
            let total_fields_num = DEFAULT_FIELDS.len() + self.fields_len() + event_fields_len;

            let mut buf = Vec::with_capacity(self.average_message_len);
            let mut serializer = serde_json::Serializer::new(&mut buf);
            let mut map_serializer = serializer.serialize_map(Some(total_fields_num)).unwrap();

            let metadata = event.metadata();

            let default_fields_size = metadata.target().len().saturating_add(DEFAULT_FIELDS_LEN);
            let event_fields_limit = self
                .max_message_len
                .saturating_sub(self.fields_len)
                .saturating_sub(default_fields_size);

            let mut visitor = JsonVisitor::new(event_fields_len, event_fields_limit);
            event.record(&mut visitor);
            let mut event_storage = visitor.storage;

            // serialize additional fields
            self.additional_fields
                .iter()
                .try_for_each(|(key, value)| map_serializer.serialize_entry(key, value))?;

            let message = event_storage.remove(MESSAGE).unwrap_or_default();
            map_serializer.serialize_entry(MSG, &message)?;

            event_storage
                .iter()
                .filter(|(key, _)| {
                    !DEFAULT_FIELDS.contains(&key.as_str())
                        && !self.additional_fields.contains_key(key.as_str())
                })
                .try_for_each(|(key, value)| map_serializer.serialize_entry(key, value))?;

            // If event under span print traceId and spanId
            if let Some((span_id, trace_id)) = ctx
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
                })
            {
                map_serializer.serialize_entry(TRACE_ID, &trace_id.to_string())?;
                map_serializer.serialize_entry(SPAN_ID, &span_id.to_string())?;
            }

            // serialize current event metadata\
            map_serializer.serialize_entry(TARGET, metadata.target())?;
            map_serializer.serialize_entry(LOG_LEVEL, metadata.level().as_str())?;

            // serialize time
            let time = time::OffsetDateTime::now_utc();
            let time_ns = time.unix_timestamp_nanos();
            map_serializer.serialize_entry(TIME, &time_ns)?;

            if let Ok(time) = time.format(
                &Iso8601::<
                    {
                        Config::DEFAULT
                            .set_time_precision(TimePrecision::Second {
                                decimal_digits: NonZeroU8::new(3),
                            })
                            .encode()
                    },
                >,
            ) {
                map_serializer.serialize_entry(TIMESTAMP, time.as_str())?;
            };

            map_serializer.end()?;
            Ok(buf)
        };

        let result: std::io::Result<Vec<u8>> = serialize();

        match result {
            Ok(formatted) => {
                write!(writer, "{}", String::from_utf8_lossy(&formatted))?;
            }
            Err(err) => {
                write!(writer, "{}", err)?;
            }
        }

        writeln!(writer)
    }
}

#[derive(Clone, Debug, Default)]
struct JsonVisitor {
    storage: BTreeMap<String, Value>,
    field_size: usize,
}

impl JsonVisitor {
    fn new(fields_num: usize, limit: usize) -> Self {
        Self {
            storage: BTreeMap::new(),
            field_size: limit.saturating_div(fields_num),
        }
    }

    fn insert<T: Serialize>(&mut self, key: impl Into<String>, value: T) {
        let mut value = serde_json::json!(value);
        limit_json_value(&mut value, self.field_size);
        self.storage.insert(key.into(), value);
    }
}

impl tracing::field::Visit for JsonVisitor {
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
                            self.insert(k.as_str(), v);
                        });
                        return;
                    }
                    None => {
                        unreachable!("Named structure should always be mapped to serde_json::Value::Object()")
                    }
                }
            }
        }

        self.insert(field.name(), serde_value)
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.insert(field.name(), value);
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.insert(field.name(), value);
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.insert(field.name(), value);
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.insert(field.name(), value);
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.insert(field.name(), value);
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.insert(field.name(), value.to_string());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        match field.name() {
            name if name.starts_with("r#") => {
                self.insert(&name[2..], format!("{:?}", value));
            }
            name => {
                self.insert(name, format!("{:?}", value));
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
                None => unreachable!("This should always be correct"),
            };

            lower_bound + new_index
        }
    }

    #[inline]
    const fn is_utf8_char_boundary(byte: u8) -> bool {
        (byte as i8) >= -0x40
    }
}

fn limit_json_value(value: &mut Value, limit: usize) {
    match value {
        Value::String(str) => {
            if str.len() > limit {
                let new_limit = round::floor_char_boundary(str, limit);
                let (limited, _) = str.split_at(new_limit);
                let _ = mem::replace(str, format!("{limited} ..."));
            }
        }
        Value::Array(array) => {
            let arr_limit = limit.saturating_div(array.len());

            for value in array {
                limit_json_value(value, arr_limit);
            }
        }
        Value::Object(object) => {
            let obj_limit = limit.saturating_div(object.len());

            for (_, value) in object.iter_mut() {
                limit_json_value(value, obj_limit);
            }
        }
        _ => {}
    }
}
