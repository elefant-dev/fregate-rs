//! Fregate [`FormatEvent`] trait implementation
use crate::error::Result;
use serde::{ser::SerializeMap, Serialize, Serializer};
use serde_json::Value;
use std::{collections::BTreeMap, fmt, num::NonZeroU8};
use time::format_description::well_known::iso8601::{Config, Iso8601, TimePrecision};
use tracing::{field::Field, Event, Subscriber};
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

const DEFAULT_FIELDS: [&str; 9] = [
    VERSION, SERVICE, COMPONENT, TARGET, MSG, LOG_LEVEL, TIME, TIMESTAMP, MESSAGE,
];

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
///     let mut formatter = EventFormatter::new();
///     formatter.add_field_to_events("test", vec![0, 1]).unwrap();
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
/// {"test":[0,1],"msg":"info message","target":"check_fregate","LogLevel":"INFO","time":1665672717498107000,"timestamp":"2022-10-13T14:51:57.498Z"}
/// {"test":[0,1],"msg":"info message","target":"check_fregate","LogLevel":"DEBUG","time":1665672717498210000,"timestamp":"2022-10-13T14:51:57.498Z"}
/// {"test":[0,1],"msg":"info message","target":"check_fregate","LogLevel":"TRACE","time":1665672717498247000,"timestamp":"2022-10-13T14:51:57.498Z"}
/// {"test":[0,1],"msg":"info message","target":"check_fregate","LogLevel":"WARN","time":1665672717498279000,"timestamp":"2022-10-13T14:51:57.498Z"}
/// ```
#[derive(Debug, Default)]
pub struct EventFormatter {
    default_fields: BTreeMap<String, Value>,
}

impl EventFormatter {
    /// Creates new [`EventFormatter`]
    pub fn new() -> Self {
        Self::default()
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
    /// ```
    pub fn add_field_to_events<V: Serialize>(&mut self, key: &str, value: V) -> Result<()> {
        if DEFAULT_FIELDS.contains(&key) {
            Err(crate::error::Error::CustomError(format!(
                "Prohibited to add key: {key} to EventFormatter"
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
        self.default_fields.insert(key.to_owned(), val);
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
        _ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let serialize = || {
            let mut buf = Vec::with_capacity(event.fields().count());
            let mut serializer = serde_json::Serializer::new(&mut buf);
            let mut map_serializer = serializer.serialize_map(None).unwrap();

            let mut visitor = JsonVisitor::default();
            event.record(&mut visitor);

            // serialize default fields
            self.default_fields
                .iter()
                .try_for_each(|(key, value)| map_serializer.serialize_entry(key, value))?;

            // serialize event fields
            let mut event_storage = visitor.storage;
            let message = event_storage.remove(MESSAGE).unwrap_or_default();
            map_serializer.serialize_entry(MSG, &message)?;

            event_storage
                .iter()
                .filter(|(key, _)| {
                    !DEFAULT_FIELDS.contains(&key.as_str())
                        && !self.default_fields.contains_key(key.as_str())
                })
                .try_for_each(|(key, value)| map_serializer.serialize_entry(key, value))?;

            // serialize current event metadata
            let metadata = event.metadata();
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
}

impl JsonVisitor {
    fn insert<T: Serialize>(&mut self, key: &str, value: T) {
        self.storage
            .insert(key.to_owned(), serde_json::json!(value));
    }
}

impl tracing::field::Visit for JsonVisitor {
    #[cfg(tracing_unstable)]
    fn record_value(&mut self, field: &Field, value: valuable::Value<'_>) {
        let serde_value = serde_json::json!(Serializable::new(value));
        let structurable = value.as_structable();

        if let Some(structurable) = structurable {
            let definition = structurable.definition();

            if definition.is_dynamic() && definition.name() == TRACING_FIELDS_STRUCTURE_NAME {
                match serde_value.as_object() {
                    Some(value) => {
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

        self.insert(field.name(), serde_value);
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

#[cfg(test)]
mod test {
    use super::*;
    use std::{
        collections::HashMap,
        io,
        sync::{Arc, Mutex},
    };
    use tracing::subscriber::with_default;
    use tracing_subscriber::fmt::format::DefaultFields;
    use tracing_subscriber::fmt::{MakeWriter, SubscriberBuilder};

    #[cfg(tracing_unstable)]
    use valuable::Valuable;

    #[cfg(tracing_unstable)]
    #[derive(Serialize, Debug, valuable_derive::Valuable)]
    pub struct MarkerTest {
        pub numnber: u32,
        pub string: String,
        pub vector: Vec<u32>,
        pub map: HashMap<u32, u32>,
    }

    #[cfg(tracing_unstable)]
    use crate::tracing_fields::TracingFields;

    #[derive(Clone, Debug)]
    struct MockWriter {
        buf: Arc<Mutex<Vec<u8>>>,
    }

    #[derive(Clone, Debug)]
    struct MockMakeWriter {
        buf: Arc<Mutex<Vec<u8>>>,
    }

    impl MockMakeWriter {
        fn new() -> Self {
            Self {
                buf: Arc::new(Mutex::new(Vec::new())),
            }
        }
        fn get_content(&self) -> String {
            let buf = self.buf.lock().unwrap();
            std::str::from_utf8(&buf[..]).unwrap().to_owned()
        }
    }

    impl MockWriter {
        fn new(buf: Arc<Mutex<Vec<u8>>>) -> Self {
            Self { buf }
        }
    }

    impl io::Write for MockWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.buf.lock().unwrap().write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.buf.lock().unwrap().flush()
        }
    }

    impl<'a> MakeWriter<'a> for MockMakeWriter {
        type Writer = MockWriter;

        fn make_writer(&'a self) -> Self::Writer {
            MockWriter::new(self.buf.clone())
        }
    }

    fn subscriber(formatter: EventFormatter) -> SubscriberBuilder<DefaultFields, EventFormatter> {
        tracing_subscriber::fmt::Subscriber::builder().event_format(formatter)
    }

    fn compare(expected: &str, actual: &str) {
        let mut actual = serde_json::from_str::<HashMap<&str, Value>>(actual).unwrap();
        let expected = serde_json::from_str::<HashMap<&str, Value>>(expected).unwrap();

        let time = actual.remove("timestamp");
        let time_naons = actual.remove("time");

        assert!(time.is_some(), "Have not found \"timestamp\" field");
        assert!(time_naons.is_some(), "Have not found \"time\" field");
        assert_eq!(actual, expected);
    }

    #[test]
    fn basic_test() {
        let mock_writer = MockMakeWriter::new();
        let subscriber = subscriber(EventFormatter::new())
            .with_writer(mock_writer.clone())
            .finish();

        with_default(subscriber, || {
            tracing::info!(check = 100, "test");
        });

        let content = mock_writer.get_content();
        let expected = "{\"check\":100,\"LogLevel\":\"INFO\",\"msg\":\"test\",\"target\":\"fregate::application::log_fmt::test\"}\n";

        compare(expected, content.as_str());
    }

    #[test]
    fn same_fields() {
        let mock_writer = MockMakeWriter::new();
        let mut formatter = EventFormatter::new();
        formatter.add_field_to_events("check", 999).unwrap();

        let subscriber = subscriber(formatter)
            .with_writer(mock_writer.clone())
            .finish();

        with_default(subscriber, || {
            tracing::info!(check = 100, "test");
        });

        let content = mock_writer.get_content();
        let expected = "{\"check\":999,\"LogLevel\":\"INFO\",\"msg\":\"test\",\"target\":\"fregate::application::log_fmt::test\"}\n";

        compare(expected, content.as_str());
    }

    #[test]
    fn default_fields() {
        let mock_writer = MockMakeWriter::new();
        let mut formatter = EventFormatter::new();

        formatter.add_field_to_events("field_1", 999).unwrap();
        formatter
            .add_field_to_events("field_2", vec![1, 2, 3, 4, 5])
            .unwrap();
        formatter.add_field_to_events("field_3", "value_3").unwrap();

        let subscriber = subscriber(formatter)
            .with_writer(mock_writer.clone())
            .finish();

        with_default(subscriber, || {
            tracing::info!("test");
        });

        let content = mock_writer.get_content();
        let expected = "{\"field_1\":999,\"field_2\":[1,2,3,4,5],\"field_3\":\"value_3\",\"LogLevel\":\"INFO\",\"msg\":\"test\",\"target\":\"fregate::application::log_fmt::test\"}\n";

        compare(expected, content.as_str());
    }

    #[test]
    #[cfg(tracing_unstable)]
    fn valuable_field() {
        let mock_writer = MockMakeWriter::new();
        let formatter = EventFormatter::new();

        let subscriber = subscriber(formatter)
            .with_writer(mock_writer.clone())
            .finish();

        #[derive(Debug, valuable_derive::Valuable)]
        struct Test {
            val: u32,
        }

        let test = Test { val: 123 };

        with_default(subscriber, || {
            tracing::info!(test = test.as_value(), "test");
        });

        let content = mock_writer.get_content();
        let expected = "{\"test\":{\"val\":123},\"LogLevel\":\"INFO\",\"msg\":\"test\",\"target\":\"fregate::application::log_fmt::test\"}\n";

        compare(expected, content.as_str());
    }

    #[test]
    #[cfg(tracing_unstable)]
    fn valuable_unnamed_structure() {
        let mock_writer = MockMakeWriter::new();
        let formatter = EventFormatter::new();

        let subscriber = subscriber(formatter)
            .with_writer(mock_writer.clone())
            .finish();

        #[derive(Debug, valuable_derive::Valuable)]
        struct Test(HashMap<u32, u32>);

        let test = Test(HashMap::from_iter([(0, 1), (1, 2)].into_iter()));

        with_default(subscriber, || {
            tracing::info!(test = test.as_value(), "test");
        });

        let content = mock_writer.get_content();
        let expected = "{\"test\":{\"0\":1,\"1\":2},\"LogLevel\":\"INFO\",\"msg\":\"test\",\"target\":\"fregate::application::log_fmt::test\"}\n";

        compare(expected, content.as_str());
    }

    #[test]
    #[cfg(tracing_unstable)]
    fn valuable_named_structure() {
        let mock_writer = MockMakeWriter::new();
        let formatter = EventFormatter::new();

        let subscriber = subscriber(formatter)
            .with_writer(mock_writer.clone())
            .finish();

        #[derive(Debug, valuable_derive::Valuable)]
        struct Test {
            named: HashMap<u32, u32>,
        }

        let test = Test {
            named: HashMap::from_iter([(0, 1), (1, 2)].into_iter()),
        };

        with_default(subscriber, || {
            tracing::info!(test = test.as_value(), "test");
        });

        let content = mock_writer.get_content();
        let expected = "{\"test\":{\"named\":{\"0\":1,\"1\":2}},\"LogLevel\":\"INFO\",\"msg\":\"test\",\"target\":\"fregate::application::log_fmt::test\"}\n";

        compare(expected, content.as_str());
    }

    #[test]
    #[cfg(tracing_unstable)]
    fn one_level_flattening() {
        let mock_writer = MockMakeWriter::new();
        let formatter = EventFormatter::new();

        let subscriber = subscriber(formatter)
            .with_writer(mock_writer.clone())
            .finish();

        #[derive(Debug, valuable_derive::Valuable)]
        struct Test {
            named: HashMap<u32, u32>,
            another: AnotherTest,
        }

        #[derive(Debug, valuable_derive::Valuable)]
        struct AnotherTest {
            named: HashMap<u32, u32>,
        }

        let test = Test {
            named: HashMap::from_iter([(0, 1), (1, 2)].into_iter()),
            another: AnotherTest {
                named: HashMap::from_iter([(0, 1), (1, 2)].into_iter()),
            },
        };

        with_default(subscriber, || {
            tracing::info!(test = test.as_value(), "test");
        });

        let content = mock_writer.get_content();
        let expected = "{\"test\":{\"named\":{\"0\":1,\"1\":2},\"another\":{\"named\":{\"0\":1,\"1\":2}}},\"LogLevel\":\"INFO\",\"msg\":\"test\",\"target\":\"fregate::application::log_fmt::test\"}\n";

        compare(expected, content.as_str());
    }

    #[test]
    #[cfg(tracing_unstable)]
    fn empty_marker() {
        let mock_writer = MockMakeWriter::new();
        let formatter = EventFormatter::new();

        let subscriber = subscriber(formatter)
            .with_writer(mock_writer.clone())
            .finish();

        let new_marker = TracingFields::new();

        with_default(subscriber, || {
            tracing::info!(marker = new_marker.as_value(), "marker_test");
        });

        let content = mock_writer.get_content();
        let expected = "{\"LogLevel\":\"INFO\",\"msg\":\"marker_test\",\"target\":\"fregate::application::log_fmt::test\"}\n";

        compare(expected, content.as_str());
    }

    #[test]
    #[cfg(tracing_unstable)]
    fn marker_test() {
        let mock_writer = MockMakeWriter::new();
        let formatter = EventFormatter::new();

        let subscriber = subscriber(formatter)
            .with_writer(mock_writer.clone())
            .finish();

        let test = MarkerTest {
            numnber: 999,
            string: "string".to_string(),
            vector: vec![1, 2, 3, 4],
            map: HashMap::from_iter([(0, 1), (2, 3)]),
        };

        let mut marker = TracingFields::with_capacity(4);
        marker.insert("number", &test.numnber);
        marker.insert("string", &test.string);
        marker.insert("vector", &test.vector);
        marker.insert("map", &test.map);
        marker.insert("random_str", &"random_str");

        with_default(subscriber, || {
            tracing::info!(marker = marker.as_value(), "marker_test");
        });

        let content = mock_writer.get_content();
        let expected = "{\"number\":999,\"string\":\"string\",\"vector\":[1,2,3,4],\"map\":{\"0\":1,\"2\":3},\"random_str\":\"random_str\",\"LogLevel\":\"INFO\",\"msg\":\"marker_test\",\"target\":\"fregate::application::log_fmt::test\"}\n";

        compare(expected, content.as_str());
    }

    #[test]
    #[should_panic]
    fn default_field_with_message() {
        EventFormatter::new()
            .add_field_to_events("message", "Hello")
            .unwrap();
    }
}
