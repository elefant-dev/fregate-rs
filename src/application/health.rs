//! Trait to implement custom Health checks
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Trait to implement custom health check which will be used to respond to health check requests
#[axum::async_trait]
pub trait Health: Send + Sync + 'static + Clone {
    /// returns [`HealthResponse`] in response to configured endpoint. By default /health.
    /// For more information see [`crate::configuration::ManagementConfig`]
    async fn alive(&self) -> Response;

    /// returns [`HealthResponse`] in response to configured endpoint. By default /ready.
    /// For more information see [`crate::configuration::ManagementConfig`]
    async fn ready(&self) -> Response;
}

/// Default structure to mark application always alive and ready.
#[derive(Default, Debug, Clone, Copy)]
pub struct AlwaysReadyAndAlive;

#[axum::async_trait]
impl Health for AlwaysReadyAndAlive {
    async fn alive(&self) -> Response {
        (StatusCode::OK, "OK").into_response()
    }

    async fn ready(&self) -> Response {
        (StatusCode::SERVICE_UNAVAILABLE, "UNAVAILABLE").into_response()
    }
}
