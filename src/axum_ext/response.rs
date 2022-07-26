use axum::body::BoxBody;
use axum::http;
use axum::http::{HeaderValue, Response};
use axum::response::IntoResponse;
use bytes::Bytes;

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
