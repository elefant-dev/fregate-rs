use crate::AppConfig;
use axum::extract::{ConnectInfo, MatchedPath};
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::IntoResponse;
use hyper::header::CONTENT_TYPE;
use hyper::Request;
use metrics::{histogram, increment_counter};
use opentelemetry::trace::SpanContext;
use opentelemetry::{global::get_text_map_propagator, trace::TraceContextExt, Context};
use opentelemetry_http::HeaderExtractor;
use std::borrow::Cow;
use std::sync::Arc;
use tokio::time::Instant;
use tracing::{info, span, Instrument, Level, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

const HEADER_GRPC_STATUS: &str = "grpc-status";
const PROTOCOL_GRPC: &str = "grpc";
const PROTOCOL_HTTP: &str = "http";
const REQ_RESP: &str = "reqresp";

#[derive(Default, Debug, Clone)]
/// Structure which contains needed for [`trace_request`] [`Span`] attributes
pub struct Attributes(Arc<Inner>);

impl Attributes {
    /// Creates new [`Attributes`] from [`AppConfig`]
    pub fn new_from_config<T>(config: &AppConfig<T>) -> Self {
        Self::new(&config.logger.service_name, &config.logger.component_name)
    }

    /// Creates new [`Attributes`]
    pub fn new(service_name: &str, component_name: &str) -> Self {
        Self(Arc::new(Inner {
            service_name: service_name.to_owned(),
            component_name: component_name.to_owned(),
        }))
    }
}

#[derive(Default, Debug, Clone)]
struct Inner {
    service_name: String,
    component_name: String,
}

/// Fn to be used with [`axum::middleware::from_fn`]
pub async fn trace_request<B>(
    req: Request<B>,
    next: Next<B>,
    attributes: Attributes,
) -> impl IntoResponse {
    if is_grpc(req.headers()) {
        trace_grpc_request(req, next, &attributes)
            .await
            .into_response()
    } else {
        trace_http_request(req, next, &attributes)
            .await
            .into_response()
    }
}

async fn trace_http_request<B>(
    request: Request<B>,
    next: Next<B>,
    attributes: &Attributes,
) -> impl IntoResponse {
    let span = make_http_span();
    let parent_context = extract_context(&request);
    span.set_parent(parent_context);

    // log request out of span
    let tracing_info = extract_tracing_info(&span);
    let req_method = request.method().to_string();
    let remote_address = extract_remote_address(&request);

    span.record("service", &attributes.0.service_name);
    span.record("component", &attributes.0.component_name);
    span.record("http.method", &req_method);
    span.record("net.peer.ip", remote_address.ip);
    span.record("net.peer.port", remote_address.port);

    let url = if let Some(matched_path) = request.extensions().get::<MatchedPath>() {
        matched_path.as_str()
    } else {
        request.uri().path()
    }
    .to_owned();

    info!(
        method = &req_method,
        url = &url,
        traceId = tracing_info.trace_id,
        spanId = tracing_info.span_id,
        ">>> [Request] [{req_method}] [{url}]"
    );

    let labels = [("protocol", PROTOCOL_HTTP), ("channel", REQ_RESP)];

    increment_counter!("traffic_count_total", &labels);
    increment_counter!("traffic_sum_total");

    let duration = Instant::now();

    let response = next.run(request).instrument(span).await;
    let elapsed = duration.elapsed();

    let duration = elapsed.as_millis();
    let duration_in_sec = elapsed.as_secs_f64();

    // log response out of span
    let status = response.status();

    let labels = [
        ("protocol", Cow::Borrowed(PROTOCOL_HTTP)),
        ("channel", Cow::Borrowed(REQ_RESP)),
        ("code", Cow::Owned(status.to_string())),
    ];

    histogram!(
        "processing_duration_seconds_sum_total",
        duration_in_sec,
        &labels
    );

    info!(
        method = &req_method,
        url = &url,
        traceId = tracing_info.trace_id,
        spanId = tracing_info.span_id,
        duration = duration,
        statusCode = status.as_str(),
        "[Response] <<< [{req_method}] [{url}] [{PROTOCOL_HTTP}] [{status}] in [{duration}ms]"
    );

    response
}

async fn trace_grpc_request<B>(
    request: Request<B>,
    next: Next<B>,
    attributes: &Attributes,
) -> impl IntoResponse {
    let span = make_grpc_span();
    let parent_context = extract_context(&request);
    span.set_parent(parent_context);

    let tracing_info = extract_tracing_info(&span);
    let req_method = request.method().to_string();
    let grpc_method = request.uri().path().to_owned();
    let remote_address = extract_remote_address(&request);

    // log request out of span
    info!(
        url = &grpc_method,
        traceId = tracing_info.trace_id,
        spanId = tracing_info.span_id,
        ">>> [Request] [{req_method}] [{grpc_method}]"
    );

    let labels = [("protocol", PROTOCOL_GRPC), ("channel", REQ_RESP)];

    span.record("service", &attributes.0.service_name);
    span.record("component", &attributes.0.component_name);
    span.record("rpc.method", &grpc_method);
    span.record("net.peer.ip", remote_address.ip);
    span.record("net.peer.port", remote_address.port);

    increment_counter!("traffic_count_total", &labels);
    increment_counter!("traffic_sum_total");

    let duration = Instant::now();
    let response = next.run(request).instrument(span).await;
    let elapsed = duration.elapsed();

    let duration = elapsed.as_millis();
    let duration_in_sec = elapsed.as_secs_f64();

    // log response out of span
    let status = extract_grpc_status(response.headers())
        .unwrap_or_default()
        .to_string();

    let labels = [
        ("protocol", Cow::Borrowed(PROTOCOL_GRPC)),
        ("channel", Cow::Borrowed(REQ_RESP)),
        ("code", Cow::Owned(status.clone())),
    ];

    histogram!(
        "processing_duration_seconds_sum_total",
        duration_in_sec,
        &labels
    );

    info!(
        url = &grpc_method,
        traceId = tracing_info.trace_id,
        spanId = tracing_info.span_id,
        duration = duration,
        statusCode = status,
        "[Response] <<< [{req_method}] [{grpc_method}] [{PROTOCOL_GRPC}] [{status}] in [{duration}ms]"
    );

    response
}

#[derive(Debug, Default, Clone)]
/// Saves ip and port as [`String`]
pub struct Address {
    ip: String,
    port: String,
}

#[derive(Debug, Default, Clone)]
/// Saves trace_id and span_id as [`String`]
pub struct TracingInfo {
    trace_id: String,
    span_id: String,
}

impl From<&SpanContext> for TracingInfo {
    fn from(ctx: &SpanContext) -> Self {
        if ctx.is_valid() {
            TracingInfo {
                trace_id: ctx.trace_id().to_string(),
                span_id: ctx.span_id().to_string(),
            }
        } else {
            Default::default()
        }
    }
}

#[cfg(not(feature = "native-tls"))]
/// Extracts remote Ip and Port from [`Request`]
pub fn extract_remote_address<B>(request: &Request<B>) -> Address {
    request
        .extensions()
        .get::<ConnectInfo<std::net::SocketAddr>>()
        .map(|ConnectInfo(addr)| Address {
            ip: addr.ip().to_string(),
            port: addr.port().to_string(),
        })
        .unwrap_or_default()
}

#[cfg(feature = "native-tls")]
/// Extracts remote Ip and Port from [`Request`]
pub fn extract_remote_address<B>(request: &Request<B>) -> Address {
    request
        .extensions()
        .get::<ConnectInfo<crate::RemoteAddr>>()
        .map(|ConnectInfo(addr)| Address {
            ip: addr.0.ip().to_string(),
            port: addr.0.port().to_string(),
        })
        .unwrap_or_default()
}

/// Extracts grpc status from [`HeaderMap`]
pub fn extract_grpc_status(headers: &HeaderMap) -> Option<u8> {
    headers
        .get(HEADER_GRPC_STATUS)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse().ok())
}

/// Extracts [`Context`] from [`Request`]
pub fn extract_context<B>(request: &Request<B>) -> Context {
    get_text_map_propagator(|propagator| propagator.extract(&HeaderExtractor(request.headers())))
}

/// Extracts [`TracingInfo`] from [`Span`]
pub fn extract_tracing_info(span: &Span) -> TracingInfo {
    let context = span.context();
    let span_ref = context.span();
    let span_context = span_ref.span_context();

    span_context.into()
}

pub(crate) fn is_grpc(headers: &HeaderMap) -> bool {
    if let Some(content_type) = headers.get(CONTENT_TYPE) {
        if content_type.as_bytes().starts_with(b"application/grpc") {
            return true;
        }
    }

    false
}

fn make_http_span() -> Span {
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

fn make_grpc_span() -> Span {
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
