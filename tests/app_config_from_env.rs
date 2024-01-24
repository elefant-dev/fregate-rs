mod app_config_from_env {
    use fregate::{bootstrap, AppConfig, ConfigSource, Empty};
    use serde::Deserialize;
    use std::net::{IpAddr, Ipv6Addr};

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    pub struct TestStruct {
        number: u32,
    }

    #[test]
    fn test_load_from() {
        std::env::set_var("TEST_HOST", "::1");
        std::env::set_var("TEST_CGROUP_METRICS", "true");
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
        std::env::set_var("TEST_LOGGING_FILE", "as213%^&*(");
        std::env::set_var("TEST_LOGGING_PATH", "./a/b/c");

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
        assert_eq!(logger.logging_file, Some("as213%^&*(".to_owned()));
        assert_eq!(logger.logging_path, Some("./a/b/c".to_owned()));
        assert_eq!(logger.msg_length, Some(0));
        assert_eq!(logger.buffered_lines_limit, Some(999));
        assert!(logger.cgroup_metrics);
    }

    #[test]
    fn negative_msg_length() {
        std::env::set_var("WRONG_NEGATIVE_LOG_MSG_LENGTH", "-123");
        let config =
            AppConfig::<Empty>::load_from([ConfigSource::EnvPrefix("WRONG_NEGATIVE")]).unwrap();
        assert!(config.observability_cfg.msg_length.is_none());
    }

    #[test]
    fn wrong_msg_length() {
        std::env::set_var("WRONG_STR_LOG_MSG_LENGTH", "1a123");
        let config = AppConfig::<Empty>::load_from([ConfigSource::EnvPrefix("WRONG_STR")]).unwrap();
        assert!(config.observability_cfg.msg_length.is_none());
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

    #[test]
    fn test_server_port_priority() {
        std::env::set_var("PLACEHOLDER_0_PORT", "1234");
        std::env::set_var("PLACEHOLDER_0_SERVER_PORT", "5678");

        let config = AppConfig::<Empty>::load_from([ConfigSource::EnvPrefix("PLACEHOLDER_0")])
            .expect("Failed to build AppConfig");

        assert_eq!(config.port, 5678);
    }

    #[test]
    fn test_server_port() {
        std::env::set_var("PLACEHOLDER_1_SERVER_PORT", "5678");

        let config = AppConfig::<Empty>::load_from([ConfigSource::EnvPrefix("PLACEHOLDER_1")])
            .expect("Failed to build AppConfig");

        assert_eq!(config.port, 5678);
    }

    #[test]
    fn test_port() {
        std::env::set_var("PLACEHOLDER_2_PORT", "5678");

        let config = AppConfig::<Empty>::load_from([ConfigSource::EnvPrefix("PLACEHOLDER_2")])
            .expect("Failed to build AppConfig");

        assert_eq!(config.port, 5678);
    }
}
