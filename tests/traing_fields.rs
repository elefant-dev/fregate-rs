mod tracing_fields {
    use fregate::observability::TracingFields;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    fn is_send(_val: impl Send) {}

    #[test]
    fn tracing_fields_is_send() {
        let mut val = TracingFields::new();

        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        let local = "local".to_owned();
        val.insert("STATIC", &local);
        val.insert(local.as_str(), &local);
        val.insert("str", &"str");
        val.insert_as_string("display_address", &socket);
        val.insert_as_debug("debug_address", &socket);

        is_send(val);
    }
}
