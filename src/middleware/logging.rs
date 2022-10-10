// FIXME(kos): Rename this to `tracing`. It seems the module is about telemetry not about logging.
use hyper::{header::HeaderMap, header::HeaderValue, Request, Response};
use metrics::{histogram, increment_counter};
use opentelemetry::{
    global::get_text_map_propagator,
    trace::{SpanId, TraceContextExt, TraceId},
    Context,
};
use opentelemetry_http::HeaderExtractor;
use std::time::Duration;
use tower_http::{
    classify::{GrpcErrorsAsFailures, ServerErrorsAsFailures, SharedClassifier},
    trace::{MakeSpan, OnRequest, OnResponse, TraceLayer},
};
use tracing::{field::display, info, span, Level, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

// FIXME(kos): Better consider this Clippy complain and refactor the type into
//             its own type alias.
/// Returns [`TraceLayer`] with basic functionality for logging incoming HTTP request and outgoing HTTP response. Creates info span on request. Uses [`ServerErrorsAsFailures`] as classifier to log errors.
#[allow(clippy::type_complexity)]
pub fn http_trace_layer() -> TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    BasicMakeSpan,
    BasicOnRequest,
    BasicOnResponse,
> {
    TraceLayer::new_for_http()
        .make_span_with(BasicMakeSpan {})
        .on_response(BasicOnResponse {})
        .on_request(BasicOnRequest {})
}

// FIXME(kos): Better consider this Clippy complain and refactor the type into
//             its own type alias.
/// Returns [`TraceLayer`] with basic functionality for logging incoming HTTP request and outgoing HTTP response. Creates info span on request. Uses [`GrpcErrorsAsFailures`] as classifier to log errors.
#[allow(clippy::type_complexity)]
pub fn grpc_trace_layer() -> TraceLayer<
    SharedClassifier<GrpcErrorsAsFailures>,
    BasicMakeSpan,
    BasicOnRequest,
    BasicOnResponse,
> {
    TraceLayer::new_for_grpc()
        .make_span_with(BasicMakeSpan {})
        .on_response(BasicOnResponse {})
        .on_request(BasicOnRequest {})
}

// FIXME(kos): If it has `Clone`, that it's natural to have `Copy` for a ZST
//             (zero-sized type).
// FIXME(kos): Braces may be removed from the definition.
/// Creates info span on incoming request
#[derive(Clone, Debug)]
pub struct BasicMakeSpan {}

impl<B> MakeSpan<B> for BasicMakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let parent_context = extract_context(request);

        let span = span!(
            Level::INFO,
            "request",
            method = tracing::field::Empty,
            uri = tracing::field::Empty
        );

        span.set_parent(parent_context);
        span
    }
}

// FIXME(kos): If it has `Clone`, that it's natural to have `Copy` for a ZST
//             (zero-sized type).
// FIXME(kos): Braces may be removed from the definition.
/// Logs message on request: "Incoming Request: method: \[{method}], uri: {uri}, x-b3-traceid: {trace_id}".
#[derive(Clone, Debug)]
pub struct BasicOnRequest {}

impl<B> OnRequest<B> for BasicOnRequest {
    fn on_request(&mut self, request: &Request<B>, span: &Span) {
        let (trace_id, span_id) = get_trace_and_span_ids(span);
        let method = request.method();
        let uri = request.uri();
        let protocol = get_protocol_type(request.headers());

        info!("Incoming Request: method: [{method}], uri: {uri}, x-b3-traceid: {trace_id}, x-b3-spanid: {span_id}");

        span.record("method", &display(method));
        span.record("uri", &display(uri));

        let labels = [
            ("protocol", protocol.to_string()),
            ("channel", "reqresp".to_string()),
        ];

        increment_counter!("traffic_count_total", &labels);
        increment_counter!("traffic_sum_total");
    }
}

// FIXME(kos): If it has `Clone`, that it's natural to have `Copy` for a ZST
//             (zero-sized type).
// FIXME(kos): Braces may be removed from the definition.
/// Logs message on response: "Outgoing Response: status code: {status}, latency: {latency}ms, x-b3-traceid: {trace_id}".
#[derive(Clone, Debug)]
pub struct BasicOnResponse {}
#[allow(clippy::expect_used)]
impl<B> OnResponse<B> for BasicOnResponse {
    fn on_response(self, response: &Response<B>, latency: Duration, span: &Span) {
        let (trace_id, span_id) = get_trace_and_span_ids(span);
        let status = response.status();
        let latency_as_millis = latency.as_millis();
        let latency_as_sec = latency.as_secs_f64();
        let protocol = get_protocol_type(response.headers());

        info!(
            "Outgoing Response: status code: {status}, latency: {latency_as_millis}ms, x-b3-traceid: {trace_id}, x-b3-spanid: {span_id}"
        );

        let labels = [
            ("protocol", protocol.to_string()),
            ("channel", "reqresp".to_string()),
            ("code", status.to_string()),
        ];

        histogram!(
            "processing_duration_seconds_sum_total",
            latency_as_sec,
            &labels
        );
    }
}

/// Extracts context from request headers
pub fn extract_context<B>(request: &Request<B>) -> Context {
    get_text_map_propagator(|propagator| propagator.extract(&HeaderExtractor(request.headers())))
}

/// Get TraceId from given [`Span`]
pub fn get_trace_and_span_ids(span: &Span) -> (TraceId, SpanId) {
    let context = span.context();
    let span_ref = context.span();
    let span_context = span_ref.span_context();

    // when logging trace_id, firstly set parent context for current span and then take from it trace_id
    // if context were invalid it will be generated, so we log correct trace_id
    if span_context.is_valid() {
        (span_context.trace_id(), span_context.span_id())
    } else {
        (TraceId::INVALID, SpanId::INVALID)
    }
}

fn get_protocol_type(headers: &HeaderMap<HeaderValue>) -> &str {
    if content_type(headers)
        .unwrap_or("invalid contant type")
        .starts_with("application/grpc")
    {
        "grpc"
    } else {
        "http"
    }
}

fn content_type(headers: &HeaderMap<HeaderValue>) -> Option<&str> {
    headers.get("content-type")?.to_str().ok()
}
