#[cfg(feature = "tls")]
mod tls {
    use fregate::{error::Error, AppConfig, Application, Empty};
    use hyper::{client::HttpConnector, Client, StatusCode, Uri};
    use hyper_rustls::{ConfigBuilderExt, HttpsConnector, HttpsConnectorBuilder};
    use rustls::{
        client::{ServerCertVerified, ServerCertVerifier},
        Certificate, ClientConfig, ServerName,
    };
    use std::{
        io::ErrorKind,
        str::FromStr,
        sync::Arc,
        time::{Duration, SystemTime},
    };
    use tokio::{sync::Mutex, time::timeout};

    const ROOTLES_PORT: u16 = 1025;
    const MAX_PORT: u16 = u16::MAX;

    const TLS_KEY_FULL_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/examples_resources/certs/tls.key"
    );
    const TLS_CERTIFICATE_FULL_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/examples_resources/certs/tls.cert"
    );

    async fn start_server() -> (u16, Duration) {
        std::env::set_var("TEST_SERVER_TLS_KEY_PATH", TLS_KEY_FULL_PATH);
        std::env::set_var("TEST_SERVER_TLS_CERT_PATH", TLS_CERTIFICATE_FULL_PATH);

        let config = Arc::new(Mutex::new(
            AppConfig::<Empty>::builder()
                .add_env_prefixed("TEST")
                .add_default()
                .build()
                .unwrap(),
        ));

        let mut free_port = None;
        let tls_timeout = config.lock().await.tls.handshake_timeout;

        for port in ROOTLES_PORT..MAX_PORT {
            config.lock().await.port = port;

            let config = config.clone();
            let application_handle = timeout(
                Duration::from_secs(1),
                tokio::task::spawn(async move {
                    let mut config = config.lock_owned().await;
                    config.port = port;
                    Application::new(&config).serve_tls().await
                }),
            )
            .await;

            match application_handle {
                Err(_elapsed) => {
                    free_port = Some(port);
                    break;
                }
                Ok(Err(err)) => {
                    panic!("Unexpected error: `{err}`.");
                }
                Ok(Ok(Err(Error::IoError(err)))) => {
                    if err.kind() == ErrorKind::AddrInUse {
                        continue;
                    } else {
                        panic!("Unexpected error: `{err}`.");
                    }
                }
                Ok(Ok(Err(err))) => {
                    panic!("Unexpected error: `{err}`.");
                }
                Ok(Ok(Ok(()))) => unreachable!("impossible"),
            }
        }

        tokio::time::sleep(Duration::from_millis(200)).await;

        (free_port.expect("No free ports are available"), tls_timeout)
    }

    fn build_client() -> Client<HttpsConnector<HttpConnector>> {
        struct DummyServerCertVerifier;
        impl ServerCertVerifier for DummyServerCertVerifier {
            fn verify_server_cert(
                &self,
                _: &Certificate,
                _: &[Certificate],
                _: &ServerName,
                _: &mut dyn Iterator<Item = &[u8]>,
                _: &[u8],
                _: SystemTime,
            ) -> Result<ServerCertVerified, rustls::Error> {
                Ok(ServerCertVerified::assertion())
            }
        }

        let mut tls = ClientConfig::builder()
            .with_safe_defaults()
            .with_native_roots()
            .with_no_client_auth();
        tls.dangerous()
            .set_certificate_verifier(Arc::new(DummyServerCertVerifier));

        let https = HttpsConnectorBuilder::new()
            .with_tls_config(tls)
            .https_only()
            .enable_http1()
            .build();

        Client::builder().http2_only(true).build(https)
    }

    #[ignore]
    #[tokio::test]
    async fn test_https_request() {
        let (port, _) = start_server().await;

        let hyper = build_client();

        let timeout = Duration::from_secs(2);
        let fut = hyper.get(Uri::from_str(&format!("https://localhost:{port}/health")).unwrap());
        let response = tokio::time::timeout(timeout, fut).await.unwrap().unwrap();

        let status = response.status();
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

        assert_eq!(StatusCode::OK, status);
        assert_eq!(body.as_ref(), b"OK");
    }

    #[ignore]
    #[tokio::test]
    async fn test_http_request() {
        let (port, tls_timeout) = start_server().await;

        let hyper = build_client();

        let timeout = tls_timeout + Duration::from_secs(2);
        let fut = hyper.get(Uri::from_str(&format!("http://localhost:{port}/health")).unwrap());
        let response = tokio::time::timeout(timeout, fut).await.unwrap();

        assert!(response.is_err());
    }
}
