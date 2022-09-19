mod app_config_from_env {
    use fregate::AppConfig;
    use serde::Deserialize;
    use std::net::{IpAddr, Ipv6Addr};

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    pub struct TestStruct {
        number: u32,
    }

    #[test]
    fn test_load_from() {
        std::env::set_var("TEST_HOST", "::1");
        std::env::set_var("TEST_PORT", "1234");
        std::env::set_var("TEST_SERVICE_NAME", "TEST");
        std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "http://0.0.0.0:4317");
        std::env::set_var("TEST_TRACE_LEVEL", "debug");
        std::env::set_var("TEST_LOG_LEVEL", "trace");
        std::env::set_var("TEST_NUMBER", "100");

        let config = AppConfig::<TestStruct>::load_from([], Some("TEST"))
            .expect("Failed to build AppConfig");

        assert_eq!(config.port, 1234);
        assert_eq!(
            config.host,
            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))
        );
        assert_eq!(config.private, TestStruct { number: 100 });

        let logger = config.logger;

        assert_eq!(
            logger.traces_endpoint,
            Some("http://0.0.0.0:4317".to_owned())
        );
        assert_eq!(logger.service_name, "TEST".to_owned());
        assert_eq!(logger.trace_level, "debug".to_owned());
        assert_eq!(logger.log_level, "trace".to_owned());
    }
}
