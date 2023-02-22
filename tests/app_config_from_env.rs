mod app_config_from_env {
    use fregate::{bootstrap, AppConfig, ConfigSource};
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
        std::env::set_var("TEST_COMPONENT_NAME", "COMPONENT_TEST");
        std::env::set_var("TEST_COMPONENT_VERSION", "1.0.0");
        std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "http://0.0.0.0:4317");
        std::env::set_var("TEST_TRACE_LEVEL", "debug");
        std::env::set_var("TEST_LOG_LEVEL", "trace");
        std::env::set_var("TEST_LOG_MSG_LENGTH", "0");
        std::env::set_var("TEST_NUMBER", "100");
        std::env::set_var("TEST_BUFFERED_LINES_LIMIT", "999");

        let config = AppConfig::<TestStruct>::load_from([ConfigSource::EnvPrefix("TEST")])
            .expect("Failed to build AppConfig");

        assert_eq!(config.port, 1234);
        assert_eq!(
            config.host,
            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))
        );
        assert_eq!(config.private, TestStruct { number: 100 });

        let logger = config.observability_cfg;

        assert_eq!(
            logger.traces_endpoint,
            Some("http://0.0.0.0:4317".to_owned())
        );
        assert_eq!(logger.service_name, "TEST".to_owned());
        assert_eq!(logger.component_name, "COMPONENT_TEST".to_owned());
        assert_eq!(logger.version, "1.0.0".to_owned());
        assert_eq!(logger.service_name, "TEST".to_owned());
        assert_eq!(logger.trace_level, "debug".to_owned());
        assert_eq!(logger.log_level, "trace".to_owned());
        assert_eq!(logger.msg_length, Some(0));
        assert_eq!(logger.buffered_lines_limit, Some(999));
    }

    #[test]
    #[should_panic]
    fn negative_msg_length() {
        std::env::set_var("WRONG_LOG_MSG_LENGTH", "-123");
        let config = AppConfig::<TestStruct>::load_from([ConfigSource::EnvPrefix("WRONG")])
            .expect("Failed to build AppConfig");

        assert_eq!(config.observability_cfg.msg_length, None);
    }

    #[test]
    #[should_panic]
    fn wrong_msg_length() {
        std::env::set_var("WRONG_LOG_MSG_LENGTH", "1a123");
        let config = AppConfig::<TestStruct>::load_from([ConfigSource::EnvPrefix("WRONG")])
            .expect("Failed to build AppConfig");

        assert_eq!(config.observability_cfg.msg_length, None);
    }

    #[tokio::test]
    async fn test_management_config_from_env() {
        std::env::set_var("MNGM_MANAGEMENT_ENDPOINTS_METRICS", "/probe/metrics");
        std::env::set_var("MNGM_MANAGEMENT_ENDPOINTS_HEALTH", "///valid");
        std::env::set_var("MNGM_MANAGEMENT_ENDPOINTS_LIVE", "invalid");
        std::env::set_var("MNGM_MANAGEMENT_ENDPOINTS_READY", "");

        let config: AppConfig =
            bootstrap([ConfigSource::EnvPrefix("MNGM")]).expect("Failed to build AppConfig");

        let management_cfg = config.management_cfg;

        assert_eq!(management_cfg.endpoints.metrics.as_ref(), "/probe/metrics");
        assert_eq!(management_cfg.endpoints.health.as_ref(), "///valid");
        assert_eq!(management_cfg.endpoints.live.as_ref(), "/live");
        assert_eq!(management_cfg.endpoints.ready.as_ref(), "/ready");
    }
}
