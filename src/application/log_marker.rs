//! ADD DOCUMENTATION
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use valuable::{Fields, NamedField, NamedValues, StructDef, Structable, Valuable, Value, Visit};

///
pub trait MarkerExt {
    ///
    fn get_log_marker(&self) -> LogMarker<'_, Self>
    where
        Self: Sized;
}

/// Log Marker
pub struct LogMarker<'a, T> {
    _source: &'a T,
    fields: HashMap<&'a str, Value<'a>>,
}

impl<'a, T> Debug for LogMarker<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("log_marker")
            .field("dummy_field", &"dummy_value")
            .finish()
    }
}

impl<'a, T> LogMarker<'a, T> {
    ///
    pub fn new(source: &'a T) -> Self {
        Self {
            _source: source,
            fields: HashMap::new(),
        }
    }

    ///
    pub fn with_capacity(source: &'a T, capacity: usize) -> Self {
        Self {
            _source: source,
            fields: HashMap::with_capacity(capacity),
        }
    }

    ///
    pub fn append<V: Valuable>(&mut self, key: &'a str, value: &'a V) {
        self.fields.insert(key, value.as_value());
    }

    ///
    pub fn append_str(&mut self, key: &'a str, value: &'a str) {
        self.fields.insert(key, Value::String(value));
    }

    ///
    pub fn remove_keys<'b>(&mut self, keys: impl IntoIterator<Item = &'b str>) {
        for key in keys {
            self.remove_by_key(key);
        }
    }

    ///
    pub fn remove_by_key(&mut self, key: &str) {
        self.fields.remove(key);
    }
}

impl<'a, T> Valuable for LogMarker<'a, T> {
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

impl<'a, T> Structable for LogMarker<'a, T> {
    fn definition(&self) -> StructDef<'_> {
        StructDef::new_dynamic("log_marker", Fields::Named(&[]))
    }
}
