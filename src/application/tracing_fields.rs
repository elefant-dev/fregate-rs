//! Instrument for logging
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use valuable::{Fields, NamedField, NamedValues, StructDef, Structable, Valuable, Value, Visit};

pub(crate) const TRACING_FIELDS_STRUCTURE_NAME: &str =
    "tracing_fields:fc848aeb-3723-438e-b3c3-35162b737a98";

/// Example:
/// This is how [`TracingFields`] is serialized to logs if used with tracing_unstable feature and [`crate::log_fmt::EventFormatter`]
///```rust
/// use fregate::{tracing_fields::TracingFields, logging::init_tracing, tokio, tracing::info};
/// use fregate::valuable::Valuable;
///
/// const STATIC: &str = "STATIC";
///
/// #[tokio::main]
/// async fn main() {
///     init_tracing("info", "info", "0.0.0", "fregate", "marker", None).unwrap();
///
///     let mut marker = TracingFields::with_capacity(10);
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
#[derive(Default)]
pub struct TracingFields<'a> {
    fields: HashMap<&'a str, Field<'a>>,
}

type Field<'a> = &'a (dyn Valuable + Send + Sync);

impl<'a> Debug for TracingFields<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("TracingFields");
        for (k, v) in self.fields.iter() {
            f.field(k, &v.as_value() as _);
        }
        f.finish()
    }
}

impl<'a> TracingFields<'a> {
    /// Creates empty [`TracingFields`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates empty [`TracingFields`] with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            fields: HashMap::with_capacity(capacity),
        }
    }

    /// Inserts a key-value pair of references into the map. If key is present its value is overwritten.
    pub fn insert<V: Valuable + Send + Sync>(&mut self, key: &'a str, value: &'a V) {
        self.fields.insert(key, value);
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

impl<'a> Valuable for TracingFields<'a> {
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

impl<'a> Structable for TracingFields<'a> {
    fn definition(&self) -> StructDef<'_> {
        StructDef::new_dynamic(TRACING_FIELDS_STRUCTURE_NAME, Fields::Named(&[]))
    }
}
