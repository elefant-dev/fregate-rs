#[cfg(feature = "tls")]
mod app_config_tls {
    use fregate::{AppConfig, Application, Empty};
    use std::time::Duration;
    use tokio::time::timeout;

    const TLS_KEY_FULL_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/examples_resources/certs/tls.key"
    );
    const TLS_CERTIFICATE_FULL_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/examples_resources/certs/tls.cert"
    );

    #[tokio::test]
    async fn tls_paths() {
        let config = AppConfig::<Empty>::default();

        assert!(config.tls.key_path.is_none());
        assert!(config.tls.cert_path.is_none());
        assert!(Application::new(config).serve_tls().await.is_err());

        std::env::set_var("TEST_SERVER_TLS_KEY_PATH", TLS_KEY_FULL_PATH);
        let config = AppConfig::<Empty>::builder()
            .add_default()
            .add_env_prefixed("TEST")
            .build()
            .unwrap();

        assert!(config.tls.key_path.is_some());
        assert!(config.tls.cert_path.is_none());
        assert!(Application::new(config).serve_tls().await.is_err());

        std::env::set_var("TEST_SERVER_TLS_CERT_PATH", TLS_CERTIFICATE_FULL_PATH);
        let config = AppConfig::<Empty>::builder()
            .add_default()
            .add_env_prefixed("TEST")
            .build()
            .unwrap();

        assert!(config.tls.key_path.is_some());
        assert!(config.tls.cert_path.is_some());
        assert!(
            timeout(Duration::from_secs(2), Application::new(config).serve_tls(),)
                .await
                .is_err()
        );
    }
}
