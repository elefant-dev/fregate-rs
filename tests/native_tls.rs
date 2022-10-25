#[cfg(feature = "native-tls")]
mod native_tls {
    use fregate::{AppConfig, Application, Empty};
    use hyper::client::HttpConnector;
    use hyper::{Body, Client, StatusCode, Uri};
    use hyper_tls::native_tls::Certificate;
    use hyper_tls::native_tls::TlsConnector;
    use hyper_tls::HttpsConnector;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
    use std::ops::Add;
    use std::str::FromStr;
    use std::time::Duration;
    use tokio::time::timeout;

    const CERTIFICATE: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/examples_resources/certs/tls.cert"
    ));

    const TLS_KEY_FULL_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/examples_resources/certs/tls.key"
    );
    const TLS_CERTIFICATE_FULL_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/examples_resources/certs/tls.cert"
    );

    fn get_free_port() -> u16 {
        let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        for p in 6000..15000 {
            if let Some(p) = test_bind_tcp(SocketAddr::new(ip_addr, p)) {
                return p;
            }
        }

        panic!("NO FREE PORTS");
    }

    fn test_bind_tcp(addr: SocketAddr) -> Option<u16> {
        Some(TcpListener::bind(addr).ok()?.local_addr().ok()?.port())
    }

    async fn start_server() -> (u16, Duration) {
        std::env::set_var("TEST_SERVER_TLS_KEY_PATH", TLS_KEY_FULL_PATH);
        std::env::set_var("TEST_SERVER_TLS_CERT_PATH", TLS_CERTIFICATE_FULL_PATH);

        let mut config = AppConfig::<Empty>::builder()
            .add_env_prefixed("TEST")
            .add_default()
            .build()
            .unwrap();

        let port = get_free_port();
        let tls_timeout = config.tls_handshake_timeout;

        tokio::task::spawn(async move {
            config.port = port;
            Application::new(&config).serve_tls().await.unwrap();
        });
        tokio::time::sleep(Duration::from_millis(200)).await;

        (port, tls_timeout)
    }

    #[tokio::test]

    async fn test_https_request() {
        let (port, _) = start_server().await;

        let mut http = HttpConnector::new();
        http.enforce_http(false);
        let certificate = Certificate::from_pem(CERTIFICATE).unwrap();

        let tls_connector = TlsConnector::builder()
            .add_root_certificate(certificate)
            .danger_accept_invalid_hostnames(true)
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
            .into();

        let https = HttpsConnector::from((http, tls_connector));
        let hyper: Client<HttpsConnector<HttpConnector>, Body> = Client::builder().build(https);

        let fut = hyper.get(Uri::from_str(&format!("https://127.0.0.1:{port}/health")).unwrap());
        let response = timeout(Duration::from_secs(2), fut).await.unwrap().unwrap();

        let status = response.status();
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

        assert_eq!(StatusCode::OK, status);
        assert_eq!(&body[..], b"OK");
    }

    #[tokio::test]

    async fn test_no_cert_request() {
        let (port, _) = start_server().await;

        let mut http = HttpConnector::new();
        http.enforce_http(false);

        let tls_connector = TlsConnector::builder().build().unwrap().into();
        let https = HttpsConnector::from((http, tls_connector));
        let hyper: Client<HttpsConnector<HttpConnector>, Body> = Client::builder().build(https);

        let fut = hyper.get(Uri::from_str(&format!("https://127.0.0.1:{port}/health")).unwrap());
        let response = timeout(Duration::from_secs(2), fut).await.unwrap();
        assert!(response.is_err())
    }

    #[tokio::test]

    async fn test_http_request() {
        let (port, tls_timeout) = start_server().await;

        let mut http = HttpConnector::new();
        http.enforce_http(false);

        let certificate = Certificate::from_pem(CERTIFICATE).unwrap();

        let tls_connector = TlsConnector::builder()
            .add_root_certificate(certificate)
            .danger_accept_invalid_hostnames(true)
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
            .into();

        let https = HttpsConnector::from((http, tls_connector));
        let hyper: Client<HttpsConnector<HttpConnector>, Body> = Client::builder().build(https);

        let fut = hyper.get(Uri::from_str(&format!("http://127.0.0.1:{port}/health")).unwrap());
        let response = timeout(tls_timeout.add(Duration::from_secs(2)), fut)
            .await
            .unwrap();

        assert!(response.is_err());
    }
}
