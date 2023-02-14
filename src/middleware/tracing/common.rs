//! Code for http/grpc requests tracing and logging.
#[cfg(feature = "tls")]
use crate::application::tls::RemoteAddr;
use axum::extract::ConnectInfo;
use axum::http::HeaderMap;
use hyper::header::CONTENT_TYPE;
use hyper::Request;
use opentelemetry::{global::get_text_map_propagator, Context};
use opentelemetry_http::HeaderExtractor;
use std::net::SocketAddr;

/// Extracts remote Ip and Port from [`Request`]
#[cfg(not(feature = "tls"))]
pub fn extract_remote_address<B>(request: &Request<B>) -> Option<&SocketAddr> {
    request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(addr)| addr)
}

/// Extracts remote Ip and Port from [`Request`]
#[cfg(feature = "tls")]
pub fn extract_remote_address<B>(request: &Request<B>) -> Option<&SocketAddr> {
    request
        .extensions()
        .get::<ConnectInfo<RemoteAddr>>()
        .map(|ConnectInfo(RemoteAddr(addr))| addr)
}

/// Extracts [`Context`] from [`Request`]
pub fn extract_context<B>(request: &Request<B>) -> Context {
    get_text_map_propagator(|propagator| propagator.extract(&HeaderExtractor(request.headers())))
}

/// Return [`true`] if incoming request [`CONTENT_TYPE`] header value starts with "application/grpc"
pub fn is_grpc(headers: &HeaderMap) -> bool {
    headers.get(CONTENT_TYPE).map_or(false, |content_type| {
        content_type.as_bytes().starts_with(b"application/grpc")
    })
}
