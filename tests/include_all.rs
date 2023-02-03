mod include_all_test {
    use fregate::extensions::HeaderFilterExt;
    use fregate::{bootstrap, ConfigSource, Empty};
    use hyper::http::HeaderValue;
    use hyper::{Body, Request};

    #[tokio::test]
    async fn include_all() {
        std::env::set_var("TEST_HEADERS_INCLUDE", "*");

        let _config = bootstrap::<Empty, _>([ConfigSource::EnvPrefix("TEST")]).unwrap();

        let request = Request::builder()
            .method("GET")
            .header("PassworD", "PasswordValue")
            .header("authorization", "authorization")
            .header("is_client", "true")
            .body(Body::empty())
            .expect("Failed to build request");

        let sanitized_headers = request.headers().get_filtered();

        assert_eq!(
            sanitized_headers.get("PassworD"),
            Some(&HeaderValue::from_str("PasswordValue").unwrap())
        );
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
