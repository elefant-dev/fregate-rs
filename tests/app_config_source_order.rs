mod app_config_source_order {
    use fregate::{AppConfig, ConfigSource, Empty};

    #[test]
    fn test_load_from() {
        std::env::set_var("TEST_PORT", "9999");

        let config =
            AppConfig::<Empty>::load_from([ConfigSource::File("./tests/resources/test_conf.toml")])
                .expect("Failed to build AppConfig");

        assert_eq!(config.port, 8888);

        let config = AppConfig::<Empty>::load_from([
            ConfigSource::File("./tests/resources/test_conf.toml"),
            ConfigSource::EnvPrefix("TEST"),
        ])
        .expect("Failed to build AppConfig");

        assert_eq!(config.port, 9999);
    }
}
