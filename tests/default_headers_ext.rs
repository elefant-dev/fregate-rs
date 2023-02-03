pub mod default_headers_ext {
    use fregate::extensions::HeaderFilterExt;
    use fregate::{bootstrap, Empty};
    use hyper::http::HeaderValue;
    use hyper::{Body, Request};

    #[tokio::test]
    async fn default_headers_ext() {
        let _config = bootstrap::<Empty, _>([]).unwrap();

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
