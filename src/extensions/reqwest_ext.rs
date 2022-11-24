use opentelemetry::global::get_text_map_propagator;
use opentelemetry_http::HeaderInjector;
use sealed::sealed;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[sealed]
/// Injects [`Span`] context into request headers;
pub trait ReqwestExt {
    /// Injects Context from [`Span::current()`]
    fn inject_from_current_span(self) -> Self;

    /// Injects Context from given [`Span`]
    fn inject_from_span(self, span: &Span) -> Self;
}

#[sealed]
impl ReqwestExt for reqwest::RequestBuilder {
    fn inject_from_current_span(self) -> Self {
        self.inject_from_span(&Span::current())
    }

    fn inject_from_span(self, span: &Span) -> Self {
        let mut headers = reqwest::header::HeaderMap::with_capacity(2);

        get_text_map_propagator(|propagator| {
            propagator.inject_context(&span.context(), &mut HeaderInjector(&mut headers))
        });

        self.headers(headers)
    }
}
