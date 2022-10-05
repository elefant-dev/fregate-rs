// FIXME(kos): Rename this to `tracing`. It seems the module is about telemetry not about logging.

use hyper::{Request, Response};
use opentelemetry::{
    global::get_text_map_propagator,
    trace::{TraceContextExt, TraceId},
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
        let trace_id = get_span_trace_id(span);
        let method = request.method();
        let uri = request.uri();

        info!("Incoming Request: method: [{method}], uri: {uri}, x-b3-traceid: {trace_id}");

        span.record("method", &display(method));
        span.record("uri", &display(uri));
    }
}

// FIXME(kos): If it has `Clone`, that it's natural to have `Copy` for a ZST
//             (zero-sized type).
// FIXME(kos): Braces may be removed from the definition.
/// Logs message on response: "Outgoing Response: status code: {status}, latency: {latency}ms, x-b3-traceid: {trace_id}".
#[derive(Clone, Debug)]
pub struct BasicOnResponse {}

impl<B> OnResponse<B> for BasicOnResponse {
    fn on_response(self, response: &Response<B>, latency: Duration, span: &Span) {
        let trace_id = get_span_trace_id(span);
        let status = response.status();
        let latency = latency.as_millis();

        info!(
            "Outgoing Response: status code: {status}, latency: {latency}ms, x-b3-traceid: {trace_id}"
        );
    }
}

/// Extracts context from request headers
pub fn extract_context<B>(request: &Request<B>) -> Context {
    get_text_map_propagator(|propagator| propagator.extract(&HeaderExtractor(request.headers())))
}

/// Get TraceId from given [`Span`]
pub fn get_span_trace_id(span: &Span) -> TraceId {
    let context = span.context();
    let span_ref = context.span();
    let span_context = span_ref.span_context();

    // when logging trace_id, firstly set parent context for current span and then take from it trace_id
    // if context were invalid it will be generated, so we log correct trace_id
    if span_context.is_valid() {
        span_context.trace_id()
    } else {
        TraceId::INVALID
    }
}
