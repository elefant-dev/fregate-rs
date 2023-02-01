mod bootstrap_fn_test {
    use fregate::logging::LOG_LAYER_HANDLE;
    use fregate::{bootstrap, Empty};
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn bootstrap_test() {
        let config = bootstrap::<Empty, _>([]).unwrap();

        assert_eq!(config.port, 8000);
        assert_eq!(config.host, IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
        assert_eq!(config.private, Empty {});

        let logger = &config.logger;
        assert_eq!(logger.traces_endpoint, None);
        assert_eq!(logger.service_name, "fregate".to_owned());
        assert_eq!(logger.version, "0.1.0".to_owned());
        assert_eq!(logger.component_name, "example".to_owned());
        assert_eq!(logger.trace_level, "info".to_owned());
        assert_eq!(logger.log_level, "info".to_owned());

        assert!(LOG_LAYER_HANDLE.get().is_some());
    }
}
