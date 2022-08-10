use hyper::{Request, Response};
use std::time::Duration;
use tower_http::classify::{GrpcErrorsAsFailures, ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{MakeSpan, OnRequest, OnResponse, TraceLayer};
use tracing::field::display;
use tracing::{info, info_span, Span};

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

#[derive(Clone, Debug)]
pub struct BasicMakeSpan {}

impl<B> MakeSpan<B> for BasicMakeSpan {
    fn make_span(&mut self, _request: &Request<B>) -> Span {
        info_span!(
            "http-request",
            method = tracing::field::Empty,
            uri = tracing::field::Empty,
        )
    }
}

#[derive(Clone, Debug)]
pub struct BasicOnRequest {}

impl<B> OnRequest<B> for BasicOnRequest {
    fn on_request(&mut self, request: &Request<B>, span: &Span) {
        let method = request.method();
        let uri = request.uri();

        info!("Incoming Request: {}, {}", method, uri);

        span.record("method", &display(method));
        span.record("uri", &display(uri));
    }
}

#[derive(Clone, Debug)]
pub struct BasicOnResponse {}

impl<B> OnResponse<B> for BasicOnResponse {
    fn on_response(self, response: &Response<B>, latency: Duration, _span: &Span) {
        info!(
            "Outgoing Response: {}, latency: {}ms",
            response.status(),
            latency.as_millis()
        );
    }
}
