use opentelemetry::global::get_text_map_propagator;
use opentelemetry::propagation::Injector;
use sealed::sealed;
use tonic::metadata::{MetadataKey, MetadataValue};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

struct MetadataMap<'a>(&'a mut tonic::metadata::MetadataMap);

impl<'a> Injector for MetadataMap<'a> {
    /// Set a key and value in the MetadataMap.  Does nothing if the key or value are not valid inputs
    fn set(&mut self, key: &str, value: String) {
        if let (Ok(key), Ok(val)) = (
            MetadataKey::from_bytes(key.as_bytes()),
            MetadataValue::try_from(value),
        ) {
            self.0.insert(key, val);
        }
    }
}

#[sealed]
/// Injects [`Span`] context into request headers;
pub trait TonicReqExt {
    /// Injects Context from [`Span::current()`]
    fn inject_from_current_span(&mut self);

    /// Injects Context from given [`Span`]
    fn inject_from_span(&mut self, span: &Span);
}

#[sealed]
impl<B> TonicReqExt for tonic::Request<B> {
    fn inject_from_current_span(&mut self) {
        self.inject_from_span(&Span::current())
    }

    fn inject_from_span(&mut self, span: &Span) {
        get_text_map_propagator(|propagator| {
            propagator.inject_context(&span.context(), &mut MetadataMap(self.metadata_mut()))
        });
    }
}
