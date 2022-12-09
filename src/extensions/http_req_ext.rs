use hyper::http;
use opentelemetry::global::get_text_map_propagator;
use opentelemetry_http::HeaderInjector;
use sealed::sealed;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[sealed]
/// Injects [`Span`] context into request headers;
pub trait HttpReqExt {
    /// Injects Context from [`Span::current()`]
    fn inject_from_current_span(&mut self);

    /// Injects Context from given [`Span`]
    fn inject_from_span(&mut self, span: &Span);
}

#[sealed]
impl<B> HttpReqExt for http::Request<B> {
    fn inject_from_current_span(&mut self) {
        self.inject_from_span(&Span::current())
    }

    fn inject_from_span(&mut self, span: &Span) {
        get_text_map_propagator(|propagator| {
            propagator.inject_context(&span.context(), &mut HeaderInjector(self.headers_mut()))
        });
    }
}
