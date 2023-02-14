mod common;
mod grpc_req;
mod http_req;

pub use common::*;
pub use grpc_req::*;
pub use http_req::*;

use axum::http::Request;
use axum::middleware::Next;
use axum::response::IntoResponse;
use tracing::Instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Fn to be used with [`axum::middleware::from_fn`]
pub async fn trace_request<B>(
    req: Request<B>,
    next: Next<B>,
    service_name: String,
    component_name: String,
) -> impl IntoResponse {
    if is_grpc(req.headers()) {
        let grpc_span = make_grpc_span();
        let parent_context = extract_context(&req);
        grpc_span.set_parent(parent_context);

        trace_grpc_request(req, next, &service_name, &component_name)
            .instrument(grpc_span)
            .await
            .into_response()
    } else {
        let http_span = make_http_span();
        let parent_context = extract_context(&req);
        http_span.set_parent(parent_context);

        trace_http_request(req, next, &service_name, &component_name)
            .instrument(http_span)
            .await
            .into_response()
    }
}
