mod exclude_one_test {
    use fregate::extensions::HeaderFilterExt;
    use fregate::{bootstrap, AppConfig, ConfigSource};
    use hyper::http::HeaderValue;
    use hyper::{Body, Request};

    #[tokio::test]
    async fn exclude_one_test() {
        std::env::set_var("TEST_HEADERS_EXCLUDE", "password");

        let _config: AppConfig = bootstrap([ConfigSource::EnvPrefix("TEST")]).unwrap();

        let request = Request::builder()
            .method("GET")
            .header("PassworD", "PasswordValue")
            .header("authorization", "authorization")
            .header("is_client", "true")
            .body(Body::empty())
            .expect("Failed to build request");

        let sanitized_headers = request.headers().get_filtered();

        assert_eq!(sanitized_headers.get("PassworD"), None);
        assert_eq!(
            sanitized_headers.get("authorization"),
            Some(&HeaderValue::from_str("authorization").unwrap())
        );
        assert_eq!(
            sanitized_headers.get("is_client"),
            Some(&HeaderValue::from_str("true").unwrap())
        );
    }
}
