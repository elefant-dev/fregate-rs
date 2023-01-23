//! Instrument for logging
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use valuable::{Fields, NamedField, NamedValues, StructDef, Structable, Valuable, Value, Visit};

pub(crate) const TRACING_FIELDS_STRUCTURE_NAME: &str =
    "tracing_fields:fc848aeb-3723-438e-b3c3-35162b737a98";

/// Example:
/// This is how [`TracingFields`] is serialized to logs if used with tracing_unstable feature and [`crate::log_fmt::EventFormatter`]
///```rust
/// use fregate::valuable::Valuable;
/// use fregate::{logging::init_tracing, tokio, tracing::info, tracing_fields::TracingFields};
/// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
///
/// const STATIC_KEY: &str = "STATIC_KEY";
/// const STATIC_VALUE: &str = "STATIC_VALUE";
///
/// #[tokio::main]
/// async fn main() {
///    let  _guard = init_tracing("info", "info", "0.0.0", "fregate", "marker", None, None, None).unwrap();
///
///    let mut marker = TracingFields::with_capacity(10);
///
///    let local_key = "LOCAL_KEY".to_owned();
///    let local_value = "LOCAL_VALUE".to_owned();
///
///    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
///
///    marker.insert(STATIC_KEY, &local_value);
///    marker.insert(&local_key, &STATIC_VALUE);
///    marker.insert_as_string("address", &socket);
///    marker.insert_as_debug("address_debug", &socket);
///
///    info!(marker = marker.as_value(), "message");
/// }
/// ```
/// Output:
///```json
/// {"component":"marker","service":"fregate","version":"0.0.0","msg":"message","LOCAL_KEY":"STATIC_VALUE","STATIC_KEY":"LOCAL_VALUE","address":"127.0.0.1:8080","address_debug":"127.0.0.1:8080","target":"playground","LogLevel":"INFO","time":1667979625296991000,"timestamp":"2022-11-09T07:40:25.297Z"}
///```
#[derive(Default)]
pub struct TracingFields<'a> {
    fields: HashMap<&'a str, Field<'a>>,
}

enum Field<'a> {
    String(String),
    ValuableRef(ValuableRef<'a>),
}

type ValuableRef<'a> = &'a (dyn Valuable + Send + Sync);

impl<'a> Debug for TracingFields<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("TracingFields");
        for (k, v) in self.fields.iter() {
            let value = match v {
                Field::String(s) => s.as_value(),
                Field::ValuableRef(r) => r.as_value(),
            };

            f.field(k, &value as _);
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
        self.fields.insert(key, Field::ValuableRef(value));
    }

    /// Converts value to [`String`] using [`Display`] implementation prior to insertion key-value pair. If key is present its value is overwritten.
    /// This will cause additional allocation.
    pub fn insert_as_string<V: Display + Sync>(&mut self, key: &'a str, value: &V) {
        self.fields.insert(key, Field::String(value.to_string()));
    }

    /// Converts value to [`String`] using [`Debug`] implementation prior to insertion key-value pair. If key is present its value is overwritten.
    /// This will cause additional allocation.
    pub fn insert_as_debug<V: Debug + Sync>(&mut self, key: &'a str, value: &V) {
        self.fields.insert(key, Field::String(format!("{value:?}")));
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
            let value_ref = match value {
                Field::String(s) => s.as_value(),
                Field::ValuableRef(r) => r.as_value(),
            };

            visit.visit_named_fields(&NamedValues::new(&[NamedField::new(field)], &[value_ref]));
        }
    }
}

impl<'a> Structable for TracingFields<'a> {
    fn definition(&self) -> StructDef<'_> {
        StructDef::new_dynamic(TRACING_FIELDS_STRUCTURE_NAME, Fields::Named(&[]))
    }
}
