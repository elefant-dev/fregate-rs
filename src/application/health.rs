//! Trait to implement custom Health checks
use axum::http::StatusCode;
use axum::response::IntoResponse;

/// Trait to implement custom health check which will be used to respond to health check requests
#[axum::async_trait]
pub trait Health: Send + Sync + 'static + Clone {
    /// return type for health check
    type HealthResponse: IntoResponse;
    /// return type for ready check
    type ReadyResponse: IntoResponse;

    /// returns [`Self::HealthResponse`] in response to configured endpoint. By default `/health`.
    /// For more information see [`crate::configuration::ManagementConfig`]
    async fn alive(&self) -> Self::HealthResponse;

    /// returns [`Self::ReadyResponse`] in response to configured endpoint. By default `/ready`.
    /// For more information see [`crate::configuration::ManagementConfig`]
    async fn ready(&self) -> Self::ReadyResponse;
}

/// Default structure to mark application always alive and ready.
#[derive(Default, Debug, Clone, Copy)]
pub struct AlwaysReadyAndAlive;

#[axum::async_trait]
impl Health for AlwaysReadyAndAlive {
    type HealthResponse = (StatusCode, &'static str);
    type ReadyResponse = (StatusCode, &'static str);

    async fn alive(&self) -> (StatusCode, &'static str) {
        (StatusCode::OK, "OK")
    }

    async fn ready(&self) -> (StatusCode, &'static str) {
        (StatusCode::OK, "OK")
    }
}
