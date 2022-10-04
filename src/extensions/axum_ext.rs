use axum::body::BoxBody;
use axum::http::{HeaderValue, Response};
use axum::response::IntoResponse;
use axum::{http, Router};
use bytes::Bytes;

// TODO(kos): Consider using crate `mime_guess`.
//            Should these functions be public?
//
//            Alternatively write more generic function.
//
//            ```rust
//            pub fn file(content: &'static str) -> Response<BoxBody> {
//                // Do the guess stuff here.
//            }
//            ```

/// Converts &str into Response and add Headers: Content-Type: "application/yaml" and "cache-control": "24 hours"
pub fn yaml(content: &'static str) -> Response<BoxBody> {
    (
        [
            (
                http::header::CONTENT_TYPE,
                HeaderValue::from_static("application/yaml"),
            ),
            // FIXME(kos): What is visibility of these function?
            //             Should them be public? If so then consider these changes.
            //             Hard-coding "Cache-Control" in this function is not
            //             very wise, because:
            //             1. doesn't allow to tune it for library users, and
            //                requires scanning all the sources, when the
            //                default value is going to be changed;
            //             2. adds unexpected subtle function semantics, as
            //                returning YAML doesn't always imply caching it for
            //                24 hours.
            //             Better provided it as a separate combinator, so
            //             caching is explicit on endpoints, when composing a
            //             router.
            (
                http::header::CACHE_CONTROL,
                HeaderValue::from_static("24 hours"),
            ),
        ],
        content,
    )
        .into_response()
}

// TODO(kos): Consider having more generic function:
//            `file(content: &Bytes, mime: &Mime) -> Response<BoxBody>`
//            and simply reuse it in this function like:
//            `file(content, mime::IMAGE_PNG)`
//            It doesn't require much effort to do so, but would be convenient
//            for library users in case they would need something other than PNG
//            for their particular case, without a need to modify this library
//            code, and preserve ergonomics this library provides.
/// Converts Bytes into Response and add Headers: Content-Type: "image/png" and "cache-control": "24 hours"
pub fn png(content: &Bytes) -> Response<BoxBody> {
    (
        [
            (
                http::header::CONTENT_TYPE,
                HeaderValue::from_static(mime::IMAGE_PNG.as_ref()),
            ),
            // FIXME(kos): Same as described in `yaml()` function above.
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
// TODO(kos): Consider name this trait as more idiomatic `RouterOptionalExt`.
// TODO(kos): Consider sealing this trait with `#[sealed]`.
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
