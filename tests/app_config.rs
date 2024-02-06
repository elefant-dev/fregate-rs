mod app_config_tests {
    use config::FileFormat;
    use fregate::{AppConfig, ConfigSource, Empty};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn multiple_config() {
        let _config = AppConfig::<Empty>::default();
        let _config = AppConfig::<Empty>::default();
    }

    #[test]
    fn default() {
        let config = AppConfig::<Empty>::default();

        assert_eq!(config.port, 8000);
        assert_eq!(config.host, IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
        assert_eq!(config.private, Empty {});

        let observ = config.observability_cfg;
        let logger = observ.logger_config;
        let mngmnt = config.management_cfg;

        assert_eq!(observ.traces_endpoint, None);
        assert_eq!(observ.service_name, "default".to_owned());
        assert_eq!(observ.version, "default".to_owned());
        assert_eq!(observ.component_name, "default".to_owned());
        assert_eq!(observ.trace_level, "info".to_owned());
        assert!(!observ.cgroup_metrics);

        assert_eq!(logger.buffered_lines_limit, None);
        assert_eq!(logger.log_level, "info".to_owned());
        assert_eq!(logger.msg_length, Some(8192));
        assert_eq!(logger.logging_file, None);
        assert_eq!(logger.logging_path, None);
        assert_eq!(logger.logging_file_limit, None);
        assert_eq!(logger.logging_file_interval, None);
        assert_eq!(logger.logging_file_max_age, None);
        assert_eq!(logger.logging_file_max_count, None);
        assert!(!logger.logging_file_enable_zip);

        assert_eq!(mngmnt.endpoints.health.as_ref(), "/health");
        assert_eq!(mngmnt.endpoints.ready.as_ref(), "/ready");
        assert_eq!(mngmnt.endpoints.live.as_ref(), "/live");
        assert_eq!(mngmnt.endpoints.metrics.as_ref(), "/metrics");
        assert_eq!(mngmnt.endpoints.version.as_ref(), "/version");
    }

    #[test]
    #[should_panic]
    fn no_file_found() {
        let _config = AppConfig::<Empty>::load_from([ConfigSource::File("")])
            .expect("Failed to build AppConfig");
    }

    #[test]
    fn empty_str_file() {
        let config = AppConfig::<Empty>::load_from([ConfigSource::String("", FileFormat::Toml)])
            .expect("Failed to build AppConfig");

        assert_eq!(config.port, 8000);
        assert_eq!(config.host, IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
        assert_eq!(config.private, Empty {});

        let observ = config.observability_cfg;

        assert_eq!(observ.traces_endpoint, None);
        assert_eq!(observ.service_name, "default".to_owned());
        assert_eq!(observ.version, "default".to_owned());
        assert_eq!(observ.component_name, "default".to_owned());
        assert_eq!(observ.trace_level, "info".to_owned());
        assert_eq!(observ.logger_config.log_level, "info".to_owned());
    }

    #[test]
    fn read_str_from_file() {
        let config = AppConfig::<Empty>::load_from([ConfigSource::String(
            include_str!("resources/test_conf.toml"),
            FileFormat::Toml,
        )])
        .expect("Failed to build AppConfig");

        assert_eq!(config.port, 8888);
        assert_eq!(
            config.host,
            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))
        );
        assert_eq!(config.private, Empty {});

        let observ = config.observability_cfg;

        assert_eq!(observ.traces_endpoint, None);
        assert_eq!(observ.service_name, "Test".to_owned());
        assert_eq!(observ.version, "default".to_owned());
        assert_eq!(observ.component_name, "default".to_owned());
        assert_eq!(observ.trace_level, "debug".to_owned());
        assert_eq!(observ.logger_config.log_level, "trace".to_owned());
    }

    #[test]
    fn read_file() {
        let config =
            AppConfig::<Empty>::load_from([ConfigSource::File("./tests/resources/test_conf.toml")])
                .expect("Failed to build AppConfig");

        assert_eq!(config.port, 8888);
        assert_eq!(
            config.host,
            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))
        );
        assert_eq!(config.private, Empty {});

        let observ = config.observability_cfg;

        assert_eq!(observ.traces_endpoint, None);
        assert_eq!(observ.service_name, "Test".to_owned());
        assert_eq!(observ.version, "default".to_owned());
        assert_eq!(observ.component_name, "default".to_owned());
        assert_eq!(observ.trace_level, "debug".to_owned());
        assert_eq!(observ.logger_config.log_level, "trace".to_owned());
    }
}
