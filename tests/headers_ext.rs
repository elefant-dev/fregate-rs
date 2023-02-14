mod headers_ext_test {
    use fregate::extensions::HeaderFilterExt;
    use fregate::observability::SANITIZED_VALUE;
    use fregate::{bootstrap, AppConfig, ConfigSource};
    use hyper::http::HeaderValue;
    use hyper::{Body, Request};

    #[tokio::test]
    async fn headers_ext_test() {
        std::env::set_var("TEST_HEADERS_INCLUDE", "authorization,password");
        std::env::set_var("TEST_HEADERS_SANITIZE", "password,authorization");
        std::env::set_var("TEST_HEADERS_EXCLUDE", "password,");

        let _config: AppConfig = bootstrap([ConfigSource::EnvPrefix("TEST")]).unwrap();

        let request = Request::builder()
            .method("GET")
            .header("PassworD", "PasswordValue")
            .header("authorization", "authorization")
            .header("is_client", "true")
            .body(Body::empty())
            .expect("Failed to build request");

        let sanitized_headers = request.headers().get_filtered();

        assert_eq!(sanitized_headers.get("PassworD"), None, "Must be Excluded");
        assert_eq!(
            sanitized_headers.get("authorization"),
            Some(&HeaderValue::from_static(SANITIZED_VALUE)),
            "Included and sanitized"
        );
        assert_eq!(sanitized_headers.get("is_client"), None, "Not included");
    }
}
