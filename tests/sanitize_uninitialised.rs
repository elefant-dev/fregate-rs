mod sanitize_uninitialised {
    use fregate::extensions::SanitizeExt;
    use fregate::{bootstrap, Empty};
    use hyper::http::HeaderValue;
    use hyper::{Body, Request};

    #[tokio::test]
    async fn headermap_test() {
        let _config = bootstrap::<Empty, _>([]).unwrap();

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
