mod tracing_fields {
    use fregate::tracing_fields::TracingFields;

    fn is_send(_val: impl Send) {}

    #[test]
    fn tracing_fields_is_send() {
        let mut val = TracingFields::new();

        let local = "local".to_owned();
        val.insert("STATIC", &local);
        val.insert(local.as_str(), &local);
        val.insert("str", &"str");

        is_send(val);
    }
}
