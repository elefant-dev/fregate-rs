use crate::middleware::extract_remote_address;
use axum::extract::MatchedPath;
use axum::middleware::Next;
use axum::response::IntoResponse;
use hyper::Request;
use tokio::time::Instant;
use tracing::{Level, Span};

const PROTOCOL_HTTP: &str = "http";

/// Fn to be used with [`axum::middleware::from_fn`] to trace http request
pub async fn trace_http_request<B>(
    request: Request<B>,
    next: Next<B>,
    service_name: &str,
    component_name: &str,
) -> impl IntoResponse {
    let span = Span::current();

    let req_method = request.method().to_string();
    let remote_address = extract_remote_address(&request);

    span.record("service", service_name);
    span.record("component", component_name);
    span.record("http.method", &req_method);

    if let Some(addr) = remote_address {
        span.record("net.peer.ip", addr.ip().to_string());
        span.record("net.peer.port", addr.port());
    }

    let url = request
        .extensions()
        .get::<MatchedPath>()
        .map_or_else(|| request.uri().path(), MatchedPath::as_str)
        .to_owned();

    tracing::info!(
        method = &req_method,
        url = &url,
        ">>> [Request] [{req_method}] [{url}]"
    );

    let duration = Instant::now();
    let response = next.run(request).await;
    let elapsed = duration.elapsed();

    let duration = elapsed.as_millis();
    let status = response.status();

    span.record("http.status_code", status.as_str());

    tracing::info!(
        method = &req_method,
        url = &url,
        duration = duration,
        statusCode = status.as_str(),
        "[Response] <<< [{req_method}] [{url}] [{PROTOCOL_HTTP}] [{status}] in [{duration}ms]"
    );

    response
}

/// Creates HTTP [`Span`] with predefined empty attributes.
pub fn make_http_span() -> Span {
    tracing::span!(
        Level::INFO,
        "http-request",
        service = tracing::field::Empty,
        component = tracing::field::Empty,
        http.method = tracing::field::Empty,
        http.status_code = tracing::field::Empty,
        net.peer.ip = tracing::field::Empty,
        net.peer.port = tracing::field::Empty,
        trace.level = "INFO"
    )
}
