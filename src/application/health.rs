//! Trait to implement custom Health checks

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Trait to implement custom health check which will be used to respond to health check requests
#[axum::async_trait]
pub trait Health: Send + Sync + 'static + Clone {
    /// return [`HealthResponse`] on /health/alive endpoint
    async fn alive(&self) -> HealthResponse;

    /// return [`HealthResponse`] on /health/ready endpoint
    async fn ready(&self) -> HealthResponse;
}

/// Variants to respond to health check request
#[derive(Debug, Clone, Copy)]
pub enum HealthResponse {
    /// returns 200 StatusCode
    OK,
    /// returns 501 StatusCode
    UNAVAILABLE,
}

impl IntoResponse for HealthResponse {
    fn into_response(self) -> Response {
        match self {
            HealthResponse::OK => (StatusCode::OK, "OK").into_response(),
            HealthResponse::UNAVAILABLE => {
                (StatusCode::SERVICE_UNAVAILABLE, "UNAVAILABLE").into_response()
            }
        }
    }
}

/// Default structure to mark application always alive and ready.
#[derive(Default, Debug, Clone)]
pub struct AlwaysReadyAndAlive {}

#[axum::async_trait]
impl Health for AlwaysReadyAndAlive {
    async fn alive(&self) -> HealthResponse {
        HealthResponse::OK
    }

    async fn ready(&self) -> HealthResponse {
        HealthResponse::UNAVAILABLE
    }
}
