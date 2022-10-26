#[cfg(any(feature = "native-tls", feature = "rustls"))]
mod tls {
    use fregate::{AppConfig, Application, Empty};
    use futures_util::{stream, StreamExt};
    use hyper::{client::HttpConnector, Client, StatusCode, Uri};
    use hyper_rustls::{ConfigBuilderExt, HttpsConnector, HttpsConnectorBuilder};
    use rustls::{
        client::{ServerCertVerified, ServerCertVerifier},
        Certificate, ClientConfig, ServerName,
    };
    use std::{
        future::ready,
        net::{IpAddr, Ipv6Addr, SocketAddr},
        str::FromStr,
        sync::Arc,
        time::{Duration, SystemTime},
    };
    use tokio::{net::TcpListener, time};

    const ROOTLES_PORT: u16 = 1024;
    const MAX_PORT: u16 = u16::MAX;

    const TLS_KEY_FULL_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/examples_resources/certs/tls.key"
    );
    const TLS_CERTIFICATE_FULL_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/examples_resources/certs/tls.cert"
    );

    async fn get_free_port() -> u16 {
        stream::iter(ROOTLES_PORT..MAX_PORT)
            .map(test_bind_tcp)
            .buffer_unordered(16)
            .filter_map(ready)
            .next()
            .await
            .expect("NO FREE PORTS")
    }

    async fn test_bind_tcp(port: u16) -> Option<u16> {
        const LOOPBACK: IpAddr = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        TcpListener::bind(SocketAddr::new(LOOPBACK, port))
            .await
            .ok()?
            .local_addr()
            .ok()
            .as_ref()
            .map(SocketAddr::port)
    }

    async fn start_server() -> (u16, Duration) {
        std::env::set_var("TEST_SERVER_TLS_KEY_PATH", TLS_KEY_FULL_PATH);
        std::env::set_var("TEST_SERVER_TLS_CERT_PATH", TLS_CERTIFICATE_FULL_PATH);

        let mut config = AppConfig::<Empty>::builder()
            .add_env_prefixed("TEST")
            .add_default()
            .build()
            .unwrap();

        let port = get_free_port().await;
        let tls_timeout = config.tls_handshake_timeout;

        tokio::task::spawn(async move {
            config.port = port;
            Application::new(&config).serve_tls().await.unwrap();
        });
        tokio::time::sleep(Duration::from_millis(200)).await;

        (port, tls_timeout)
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
        let response = time::timeout(timeout, fut).await.unwrap().unwrap();

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
        let response = time::timeout(timeout, fut).await.unwrap();

        assert!(response.is_err());
    }
}
