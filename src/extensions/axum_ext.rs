use axum::body::BoxBody;
use axum::http::{HeaderValue, Response};
use axum::response::IntoResponse;
use axum::{http, Router};
use bytes::Bytes;

// TODO(kos): Consider using crate `mime_guess`.
/// Converts &str into Response and add Headers: Content-Type: "application/yaml" and "cache-control": "24 hours"
pub fn yaml(content: &'static str) -> Response<BoxBody> {
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

/// Converts Bytes into Response and add Headers: Content-Type: "image/png" and "cache-control": "24 hours"
pub fn png(content: &Bytes) -> Response<BoxBody> {
    (
        [
            (
                http::header::CONTENT_TYPE,
                HeaderValue::from_static(mime::IMAGE_PNG.as_ref()),
            ),
            (
                http::header::CACHE_CONTROL,
                HeaderValue::from_static("24 hours"),
            ),
        ],
        content.to_vec(),
    )
        .into_response()
}

//TODO: Might be substituted with Router::nest(other.unwrap_or_default())
/// Used to merge and nest Option<Router>
pub trait Optional {
    /// Used to merge Option<Router>
    fn merge_optional(self, other: Option<Router>) -> Self;

    /// Used to nest Option<Router>
    fn nest_optional(self, path: &str, other: Option<Router>) -> Self;
}

impl Optional for Router {
    fn merge_optional(self, mut other: Option<Router>) -> Self {
        if let Some(other) = other.take() {
            self.merge(other)
        } else {
            self
        }
    }

    fn nest_optional(self, path: &str, mut other: Option<Router>) -> Self {
        if let Some(other) = other.take() {
            self.nest(path, other)
        } else {
            self
        }
    }
}
