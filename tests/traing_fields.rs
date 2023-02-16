mod tracing_fields {
    use fregate::observability::TracingFields;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    fn is_send(_val: impl Send) {}

    #[test]
    fn tracing_fields_is_send() {
        let mut val = TracingFields::new();

        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        val.insert("str", &"str");
        val.insert_as_string("display_address", &socket);
        val.insert_as_debug("debug_address", &socket);

        is_send(val);
    }

    #[test]
    fn owning_data() {
        let mut val = TracingFields::new();

        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080).to_string();

        val.insert("str", &"str");
        val.insert_as_string("1", &socket);
        val.insert_as_debug("2", &socket);
        val.insert_as_owned("3", socket.to_string());

        drop(socket);
        is_send(val);
    }
}
