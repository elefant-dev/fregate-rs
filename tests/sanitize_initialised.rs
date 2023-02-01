mod sanitize_initialised {
    use fregate::extensions::SanitizeExt;
    use fregate::logging::SANITIZED_VALUE;
    use fregate::{bootstrap, ConfigSource, Empty};
    use hyper::http::HeaderValue;
    use hyper::{Body, Request};

    #[tokio::test]
    async fn headermap_test() {
        std::env::set_var("TEST_SANITIZE_FIELDS", "password,check,authorization");

        let _config = bootstrap::<Empty, _>([ConfigSource::EnvPrefix("TEST")]).unwrap();

        let request = Request::builder()
            .method("GET")
            .header("PassworD", "PasswordValue")
            .header("authorization", "authorization")
            .header("is_client", "true")
            .body(Body::empty())
            .expect("Failed to build request");

        let sanitized_headers = request.headers().get_sanitized();

        assert_eq!(
            sanitized_headers.get("PassworD"),
            Some(&HeaderValue::from_static(SANITIZED_VALUE))
        );
        assert_eq!(
            sanitized_headers.get("authorization"),
            Some(&HeaderValue::from_static(SANITIZED_VALUE))
        );
        assert_eq!(
            sanitized_headers.get("is_client"),
            Some(&HeaderValue::from_str("true").unwrap())
        );
    }
}
