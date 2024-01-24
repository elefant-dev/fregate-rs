//! Instrument for logging
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use valuable::{Fields, NamedField, NamedValues, StructDef, Structable, Valuable, Value, Visit};

pub(crate) const TRACING_FIELDS_STRUCTURE_NAME: &str =
    "tracing_fields:fc848aeb-3723-438e-b3c3-35162b737a98";

/// If used in pair with [`crate::observability::EventFormatter`] and unstable [`tracing feature`](https://github.com/tokio-rs/tracing/discussions/1906) this has custom logging behaviour. See in example below:\
/// Once feature is stabilised this might change.
///
/// Example:
///```rust
/// use fregate::observability::init_tracing;
/// use fregate::observability::TracingFields;
/// use fregate::valuable::Valuable;
/// use fregate::{tokio, tracing::info};
/// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
///
/// const STATIC_KEY: &str = "STATIC_KEY";
/// const STATIC_VALUE: &str = "STATIC_VALUE";
///
/// #[tokio::main]
/// async fn main() {
///     let _guard = init_tracing(
///         "info", "info", "0.0.0", "fregate", "marker", None, None, None, None, None, None
///     ).unwrap();
///
///     let mut marker = TracingFields::with_capacity(10);
///     let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
///
///     marker.insert_ref(STATIC_KEY, &STATIC_VALUE);
///     marker.insert_as_string("address", &socket);
///     marker.insert_as_debug("address_debug", &socket);
///
///     info!(marker = marker.as_value(), "message");
/// }
///
/// ```
/// Output:
///```json
/// {"time":1676630313178421000,"timestamp":"2023-02-17T10:38:33.178Z","LogLevel":"INFO","target":"playground","component":"marker","service":"fregate","version":"0.0.0","msg":"message","STATIC_KEY":"STATIC_VALUE","address":"127.0.0.1:8080","address_debug":"127.0.0.1:8080"}
///```
#[derive(Default)]
pub struct TracingFields<'a> {
    fields: HashMap<&'static str, Field<'a>>,
}

enum Field<'a> {
    Str(&'a str),
    String(String),
    ValuableRef(&'a (dyn Valuable + Send + Sync)),
    ValuableOwned(Box<dyn Valuable + Send + Sync>),
}

impl<'a> Field<'a> {
    fn as_value(&self) -> Value<'_> {
        match self {
            Field::Str(s) => s.as_value(),
            Field::String(s) => s.as_value(),
            Field::ValuableOwned(s) => s.as_value(),
            Field::ValuableRef(r) => r.as_value(),
        }
    }
}

impl<'a> Debug for TracingFields<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("TracingFields");
        for (k, v) in self.fields.iter() {
            let value = v.as_value();

            f.field(k, &value as _);
        }
        f.finish()
    }
}

impl<'a> TracingFields<'a> {
    /// Create empty [`TracingFields`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Create empty [`TracingFields`] with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            fields: HashMap::with_capacity(capacity),
        }
    }

    /// Wraps value into [`Box`] and insert key-value pair into the map. Overwrites value if key is present.
    pub fn insert<V: Valuable + Send + Sync + 'static>(&mut self, key: &'static str, value: V) {
        self.fields
            .insert(key, Field::ValuableOwned(Box::new(value)));
    }

    /// Inserts a key-value pair of references into the map. Overwrites value if key is present.
    pub fn insert_ref<V: Valuable + Send + Sync>(&mut self, key: &'static str, value: &'a V) {
        self.fields.insert(key, Field::ValuableRef(value));
    }

    /// Inserts a key-value pair of references into the map. Overwrites value if key is present.
    pub fn insert_str(&mut self, key: &'static str, value: &'a str) {
        self.fields.insert(key, Field::Str(value));
    }

    /// Converts value to [`String`] using [`Display`] implementation before insertion. Overwrites value if key is present.
    pub fn insert_as_string<V: Display + Sync>(&mut self, key: &'static str, value: &V) {
        self.fields.insert(key, Field::String(value.to_string()));
    }

    /// Converts value to [`String`] using [`Debug`] implementation before insertion. Overwrites value if key is present.
    pub fn insert_as_debug<V: Debug + Sync>(&mut self, key: &'static str, value: &V) {
        self.fields.insert(key, Field::String(format!("{value:?}")));
    }

    /// Removes each key from the map.
    pub fn remove_keys<'b>(&mut self, keys: impl IntoIterator<Item = &'b str>) {
        for key in keys {
            self.remove_by_key(key);
        }
    }

    /// Removes key from the map.
    pub fn remove_by_key(&mut self, key: &str) {
        self.fields.remove(key);
    }

    /// Merge with other [`TracingFields`] consuming other.
    pub fn merge<'b: 'a>(&mut self, other: TracingFields<'b>) {
        self.fields.reserve(other.fields.len());

        other.fields.into_iter().for_each(|(k, v)| {
            self.fields.insert(k, v);
        });
    }
}

impl<'a> Valuable for TracingFields<'a> {
    fn as_value(&self) -> Value<'_> {
        Value::Structable(self)
    }

    fn visit(&self, visit: &mut dyn Visit) {
        for (field, value) in self.fields.iter() {
            let value_ref = value.as_value();

            visit.visit_named_fields(&NamedValues::new(&[NamedField::new(field)], &[value_ref]));
        }
    }
}

impl<'a> Structable for TracingFields<'a> {
    fn definition(&self) -> StructDef<'_> {
        StructDef::new_dynamic(TRACING_FIELDS_STRUCTURE_NAME, Fields::Named(&[]))
    }
}

#[cfg(test)]
mod test {
    use crate::observability::TracingFields;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn test_merge_with_static() {
        let mut destination = TracingFields::new();
        let mut source = TracingFields::new();

        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080).to_string();

        destination.insert_ref("static", &"static");
        source.insert_ref("socket", &socket);

        destination.merge(source);

        assert_eq!(destination.fields.len(), 2);
        assert!(destination.fields.contains_key("static"));
        assert!(destination.fields.contains_key("socket"));
    }

    #[test]
    fn test_merge_with_local() {
        let mut destination = TracingFields::new();
        let mut source = TracingFields::new();

        let first = "first".to_owned();
        let second = "second".to_owned();

        destination.insert_ref("first", &first);
        source.insert_ref("second", &second);

        destination.merge(source);

        assert_eq!(destination.fields.len(), 2);
        assert!(destination.fields.contains_key("first"));
        assert!(destination.fields.contains_key("second"));
    }
}
