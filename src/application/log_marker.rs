//! Instrument for logging
use std::collections::HashMap;
use std::fmt::Debug;
use valuable::{Fields, NamedField, NamedValues, StructDef, Structable, Valuable, Value, Visit};

pub(crate) const LOG_MARKER_STRUCTURE_NAME: &str =
    "log_marker:fc848aeb-3723-438e-b3c3-35162b737a98";

/// Example:
/// This is how [`LogMarker`] is serialized to logs if used with tracing_unstable feature and [`crate::log_fmt::EventFormatter`]
///```rust
/// use fregate::{log_marker::LogMarker, logging::init_tracing, tokio, tracing::info};
/// use fregate::valuable::Valuable;
///
/// const STATIC: &str = "STATIC";
///
/// #[tokio::main]
/// async fn main() {
///     init_tracing("info", "info", "0.0.0", "fregate", "marker", None).unwrap();
///
///     let mut marker = LogMarker::with_capacity(10);
///     let local_key = "NON_STATIC".to_owned();
///     let local_var = 1000;
///
///     marker.insert(STATIC, &local_var);
///     marker.insert(local_key.as_str(), &local_var);
///     marker.insert("str", &"str");
///
///     info!(marker = marker.as_value(), "message");
/// }
/// ```
/// Output:
///```json
///  {"component":"marker","service":"fregate","version":"0.0.0","NON_STATIC":1000,"STATIC":1000,"str":"str","msg":"message","target":"check_fregate","LogLevel":"INFO","time":1665656359172240000,"timestamp":"2022-10-13T10:19:19.172Z"}
///```
#[derive(Debug, Default)]
pub struct LogMarker<'a> {
    fields: HashMap<&'a str, Value<'a>>,
}

impl<'a> LogMarker<'a> {
    /// Creates empty [`LogMarker`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates empty [`LogMarker`] with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            fields: HashMap::with_capacity(capacity),
        }
    }

    /// Inserts a key-value pair of references into the map. If key is present its value is overwritten.
    pub fn insert<V: Valuable>(&mut self, key: &'a str, value: &'a V) {
        self.fields.insert(key, value.as_value());
    }

    /// Removes key-value pairs from the by given keys.
    pub fn remove_keys<'b>(&mut self, keys: impl IntoIterator<Item = &'b str>) {
        for key in keys {
            self.remove_by_key(key);
        }
    }

    /// Removes key-value pair by key.
    pub fn remove_by_key(&mut self, key: &str) {
        self.fields.remove(key);
    }
}

impl<'a> Valuable for LogMarker<'a> {
    fn as_value(&self) -> Value<'_> {
        Value::Structable(self)
    }

    fn visit(&self, visit: &mut dyn Visit) {
        for (field, value) in self.fields.iter() {
            visit.visit_named_fields(&NamedValues::new(
                &[NamedField::new(field)],
                &[value.as_value()],
            ));
        }
    }
}

impl<'a> Structable for LogMarker<'a> {
    fn definition(&self) -> StructDef<'_> {
        StructDef::new_dynamic(LOG_MARKER_STRUCTURE_NAME, Fields::Named(&[]))
    }
}
