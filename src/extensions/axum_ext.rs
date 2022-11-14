use axum::body::BoxBody;
use axum::http::{HeaderValue, Response};
use axum::response::IntoResponse;
use axum::{http, Router};
use sealed::sealed;

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

//TODO: Might be substituted with Router::nest(other.unwrap_or_default())
/// Used to merge and nest Option<Router>
#[sealed]
pub trait RouterOptionalExt {
    /// Used to merge Option<Router>
    fn merge_optional(self, other: Option<Router>) -> Self;

    /// Used to nest Option<Router>
    fn nest_optional(self, path: &str, other: Option<Router>) -> Self;
}

#[sealed]
impl RouterOptionalExt for Router {
    fn merge_optional(self, other: Option<Router>) -> Self {
        if let Some(other) = other {
            self.merge(other)
        } else {
            self
        }
    }

    fn nest_optional(self, path: &str, other: Option<Router>) -> Self {
        if let Some(other) = other {
            self.nest(path, other)
        } else {
            self
        }
    }
}
