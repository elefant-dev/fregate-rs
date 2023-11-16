use crate::extensions::HttpReqExt;
use crate::middleware::tracing::common::extract_remote_address;
use axum::middleware::Next;
use axum::response::IntoResponse;
use hyper::header::HeaderValue;
use hyper::{HeaderMap, Request};
use std::str::FromStr;
use tokio::time::Instant;
use tracing::{Level, Span};

const HEADER_GRPC_STATUS: &str = "grpc-status";
const PROTOCOL_GRPC: &str = "grpc";

/// Fn to be used with [`axum::middleware::from_fn`] to trace grpc request
pub async fn trace_grpc_request<B>(
    request: Request<B>,
    next: Next<B>,
    service_name: &str,
    component_name: &str,
) -> impl IntoResponse {
    let span = Span::current();

    let req_method = request.method().to_string();
    let grpc_method = request.uri().path().to_owned();
    let remote_address = extract_remote_address(&request);

    tracing::info!(
        url = &grpc_method,
        ">>> [Request] [{req_method}] [{grpc_method}]"
    );

    span.record("service", service_name);
    span.record("component", component_name);
    span.record("rpc.method", &grpc_method);

    if let Some(addr) = remote_address {
        span.record("net.peer.ip", addr.ip().to_string());
        span.record("net.peer.port", addr.port());
    }

    let duration = Instant::now();
    let mut response = next.run(request).await;
    let elapsed = duration.elapsed();

    let duration = elapsed.as_millis();

    let status: i32 = extract_grpc_status_code(response.headers())
        .unwrap_or(tonic::Code::Unknown)
        .into();

    span.record("rpc.grpc.status_code", status);

    tracing::info!(
        url = &grpc_method,
        duration = duration,
        statusCode = status,
        "[Response] <<< [{req_method}] [{grpc_method}] [{PROTOCOL_GRPC}] [{status}] in [{duration}ms]"
    );

    response.headers_mut().inject_from_current_span();
    response
}

/// Creates GRPC [`Span`] with predefined empty attributes.
pub fn make_grpc_span() -> Span {
    tracing::span!(
        Level::INFO,
        "grpc-request",
        service = tracing::field::Empty,
        component = tracing::field::Empty,
        rpc.system = "grpc",
        rpc.method = tracing::field::Empty,
        rpc.grpc.stream.id = tracing::field::Empty,
        rpc.grpc.status_code = tracing::field::Empty,
        net.peer.ip = tracing::field::Empty,
        net.peer.port = tracing::field::Empty,
        trace.level = "INFO"
    )
}
/// Extracts grpc status from [`HeaderMap`]
pub fn extract_grpc_status_code(headers: &HeaderMap) -> Option<tonic::Code> {
    headers
        .get(HEADER_GRPC_STATUS)
        .map(HeaderValue::to_str)
        .and_then(Result::ok)
        .map(i32::from_str)
        .and_then(Result::ok)
        .map(tonic::Code::from)
}
