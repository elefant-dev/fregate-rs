//! Code for http/grpc requests tracing and logging.
use axum::extract::{ConnectInfo, MatchedPath};
use axum::http::{HeaderMap, HeaderValue};
use axum::middleware::Next;
use axum::response::IntoResponse;
use hyper::header::CONTENT_TYPE;
use hyper::Request;
use opentelemetry::{global::get_text_map_propagator, Context};
use opentelemetry_http::HeaderExtractor;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::time::Instant;
use tracing::{info, span, Level, Span};

const HEADER_GRPC_STATUS: &str = "grpc-status";
const PROTOCOL_GRPC: &str = "grpc";
const PROTOCOL_HTTP: &str = "http";

#[derive(Default, Debug, Clone)]
/// Configuration for Default tracing Layer.
pub struct TraceRequestConfig {
    service_name: String,
    component_name: String,
}

impl TraceRequestConfig {
    /// Set component name to be present in Spans and Logs
    #[must_use]
    pub fn component_name(self, component_name: &str) -> Self {
        Self {
            service_name: component_name.to_owned(),
            ..self
        }
    }

    /// Set service name to be present in Spans and Logs
    #[must_use]
    pub fn service_name(self, service_name: &str) -> Self {
        Self {
            service_name: service_name.to_owned(),
            ..self
        }
    }
}

/// Fn to be used with [`axum::middleware::from_fn`] to trace http request
pub async fn trace_http_request<B>(
    request: Request<B>,
    next: Next<B>,
    config: &TraceRequestConfig,
) -> impl IntoResponse {
    let span = Span::current();

    let req_method = request.method().to_string();
    let remote_address = extract_remote_address(&request);

    span.record("service", &config.service_name);
    span.record("component", &config.component_name);
    span.record("http.method", &req_method);
    span.record("net.peer.ip", remote_address.ip);
    span.record("net.peer.port", remote_address.port);

    let url = request
        .extensions()
        .get::<MatchedPath>()
        .map_or_else(|| request.uri().path(), MatchedPath::as_str)
        .to_owned();

    info!(
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

    info!(
        method = &req_method,
        url = &url,
        duration = duration,
        statusCode = status.as_str(),
        "[Response] <<< [{req_method}] [{url}] [{PROTOCOL_HTTP}] [{status}] in [{duration}ms]"
    );

    response
}

/// Fn to be used with [`axum::middleware::from_fn`] to trace grpc request
pub async fn trace_grpc_request<B>(
    request: Request<B>,
    next: Next<B>,
    config: &TraceRequestConfig,
) -> impl IntoResponse {
    let span = Span::current();

    let req_method = request.method().to_string();
    let grpc_method = request.uri().path().to_owned();
    let remote_address = extract_remote_address(&request);

    info!(
        url = &grpc_method,
        ">>> [Request] [{req_method}] [{grpc_method}]"
    );

    span.record("service", &config.service_name);
    span.record("component", &config.component_name);
    span.record("rpc.method", &grpc_method);
    span.record("net.peer.ip", remote_address.ip);
    span.record("net.peer.port", remote_address.port);

    let duration = Instant::now();
    let response = next.run(request).await;
    let elapsed = duration.elapsed();

    let duration = elapsed.as_millis();

    let status: i32 = extract_grpc_status_code(response.headers())
        .unwrap_or(tonic::Code::Unknown)
        .into();

    span.record("rpc.grpc.status_code", status);

    info!(
        url = &grpc_method,
        duration = duration,
        statusCode = status,
        "[Response] <<< [{req_method}] [{grpc_method}] [{PROTOCOL_GRPC}] [{status}] in [{duration}ms]"
    );

    response
}

#[derive(Debug, Default, Clone)]
/// Saves ip and port to [`String`]
pub struct Address {
    /// Ip
    pub ip: String,
    /// Port
    pub port: String,
}

/// Extracts remote Ip and Port from [`Request`]
pub fn extract_remote_address<B>(request: &Request<B>) -> Address {
    request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(addr)| Address {
            ip: addr.ip().to_string(),
            port: addr.port().to_string(),
        })
        .unwrap_or_default()
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

/// Extracts [`Context`] from [`Request`]
pub fn extract_context<B>(request: &Request<B>) -> Context {
    get_text_map_propagator(|propagator| propagator.extract(&HeaderExtractor(request.headers())))
}

/// Return [`true`] if incoming Request is grpc by checking if [`CONTENT_TYPE`] header value starts with "application/grpc"
pub fn is_grpc(headers: &HeaderMap) -> bool {
    headers.get(CONTENT_TYPE).map_or(false, |content_type| {
        content_type.as_bytes().starts_with(b"application/grpc")
    })
}

/// Creates HTTP [`Span`] with predefined empty attributes.
pub fn make_http_span() -> Span {
    span!(
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

/// Creates GRPC [`Span`] with predefined empty attributes.
pub fn make_grpc_span() -> Span {
    span!(
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
