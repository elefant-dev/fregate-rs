mod log_fmt_test {
    use serde::Serialize;
    use serde_json::Value;
    use std::{
        collections::HashMap,
        io,
        sync::{Arc, Mutex},
    };
    use tracing::subscriber::with_default;
    use tracing_subscriber::fmt::format::DefaultFields;
    use tracing_subscriber::fmt::{MakeWriter, SubscriberBuilder};

    use fregate::observability::EventFormatter;
    use fregate::observability::TracingFields;
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
        let expected = "{\"check\":100,\"LogLevel\":\"INFO\",\"msg\":\"test\",\"target\":\"log_fmt::log_fmt_test\"}\n";

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
        let expected = "{\"check\":999,\"LogLevel\":\"INFO\",\"msg\":\"test\",\"target\":\"log_fmt::log_fmt_test\"}\n";

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
        let expected = "{\"field_1\":999,\"field_2\":[1,2,3,4,5],\"field_3\":\"value_3\",\"LogLevel\":\"INFO\",\"msg\":\"test\",\"target\":\"log_fmt::log_fmt_test\"}\n";

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
        let expected = "{\"test\":{\"val\":123},\"LogLevel\":\"INFO\",\"msg\":\"test\",\"target\":\"log_fmt::log_fmt_test\"}\n";

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
        let expected = "{\"test\":{\"0\":1,\"1\":2},\"LogLevel\":\"INFO\",\"msg\":\"test\",\"target\":\"log_fmt::log_fmt_test\"}\n";

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
        let expected = "{\"test\":{\"named\":{\"0\":1,\"1\":2}},\"LogLevel\":\"INFO\",\"msg\":\"test\",\"target\":\"log_fmt::log_fmt_test\"}\n";

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
        let expected = "{\"test\":{\"named\":{\"0\":1,\"1\":2},\"another\":{\"named\":{\"0\":1,\"1\":2}}},\"LogLevel\":\"INFO\",\"msg\":\"test\",\"target\":\"log_fmt::log_fmt_test\"}\n";

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
        let expected = "{\"LogLevel\":\"INFO\",\"msg\":\"marker_test\",\"target\":\"log_fmt::log_fmt_test\"}\n";

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
        let expected = "{\"number\":999,\"string\":\"string\",\"vector\":[1,2,3,4],\"map\":{\"0\":1,\"2\":3},\"random_str\":\"random_str\",\"LogLevel\":\"INFO\",\"msg\":\"marker_test\",\"target\":\"log_fmt::log_fmt_test\"}\n";

        compare(expected, content.as_str());
    }

    #[test]
    #[should_panic]
    fn additional_message_field() {
        EventFormatter::new()
            .add_field_to_events("message", "Hello")
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn additional_msg_field() {
        EventFormatter::new()
            .add_field_to_events("msg", "Hello")
            .unwrap();
    }

    #[test]
    fn limit_exact_message_size() {
        let mock_writer = MockMakeWriter::new();
        let formatter = EventFormatter::new_with_limit(Some(10));

        let subscriber = subscriber(formatter)
            .with_writer(mock_writer.clone())
            .finish();

        with_default(subscriber, || {
            tracing::info!("1234567890");
        });

        let content = mock_writer.get_content();
        let expected =
            "{\"msg\":\"1234567890\",\"target\":\"log_fmt::log_fmt_test\",\"LogLevel\":\"INFO\"}\n";

        compare(expected, content.as_str());
    }

    #[test]
    fn limit_1_byte_overflow() {
        let mock_writer = MockMakeWriter::new();
        let formatter = EventFormatter::new_with_limit(Some(10));

        let subscriber = subscriber(formatter)
            .with_writer(mock_writer.clone())
            .finish();

        with_default(subscriber, || {
            tracing::info!("12345678901");
        });

        let content = mock_writer.get_content();
        let expected =
            "{\"msg\":\"1234567890 ...\",\"target\":\"log_fmt::log_fmt_test\",\"LogLevel\":\"INFO\"}\n";

        compare(expected, content.as_str());
    }

    #[test]
    fn no_limit() {
        let mock_writer = MockMakeWriter::new();
        let formatter = EventFormatter::new();

        let subscriber = subscriber(formatter)
            .with_writer(mock_writer.clone())
            .finish();

        with_default(subscriber, || {
            tracing::info!("1234567890");
        });

        let content = mock_writer.get_content();
        let expected =
            "{\"msg\":\"1234567890\",\"target\":\"log_fmt::log_fmt_test\",\"LogLevel\":\"INFO\"}\n";

        compare(expected, content.as_str());
    }

    #[test]
    fn limit_large_message() {
        let mock_writer = MockMakeWriter::new();
        let formatter = EventFormatter::new_with_limit(Some(10));

        let subscriber = subscriber(formatter)
            .with_writer(mock_writer.clone())
            .finish();

        let data = unsafe { String::from_utf8_unchecked(vec![b'X'; 3000]) };
        with_default(subscriber, || {
            tracing::info!("{data}");
        });

        let content = mock_writer.get_content();
        let expected =
            "{\"msg\":\"XXXXXXXXXX ...\",\"target\":\"log_fmt::log_fmt_test\",\"LogLevel\":\"INFO\"}\n";

        compare(expected, content.as_str());
    }

    #[test]
    fn limit_empty_message() {
        let mock_writer = MockMakeWriter::new();
        let formatter = EventFormatter::new_with_limit(Some(10));

        let subscriber = subscriber(formatter)
            .with_writer(mock_writer.clone())
            .finish();

        with_default(subscriber, || {
            tracing::info!("");
        });

        let content = mock_writer.get_content();
        let expected =
            "{\"msg\":\"\",\"target\":\"log_fmt::log_fmt_test\",\"LogLevel\":\"INFO\"}\n";

        compare(expected, content.as_str());
    }
}
