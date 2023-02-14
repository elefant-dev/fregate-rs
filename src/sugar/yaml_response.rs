use axum::body::BoxBody;
use axum::http;
use axum::http::{HeaderValue, Response};
use axum::response::IntoResponse;

/// Converts &str into Response and add Headers: Content-Type: "application/yaml" and "cache-control": "24 hours"
pub(crate) async fn yaml(content: &'static str) -> Response<BoxBody> {
    (
        [
            (
                http::header::CONTENT_TYPE,
                HeaderValue::from_static("application/yaml"),
            ),
            (
                http::header::CACHE_CONTROL,
                HeaderValue::from_static("24 hours"),
            ),
        ],
        content,
    )
        .into_response()
}
