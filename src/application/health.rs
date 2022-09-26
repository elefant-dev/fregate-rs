use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Trait to implement custom health check which will be used to respond to health check requests
#[axum::async_trait]
pub trait Health: Send + Sync + 'static + Clone {
    /// return [`ApplicationStatus`] on /health/alive endpoint
    async fn alive(&self) -> ApplicationStatus;

    /// return [`ApplicationStatus`] on /health/ready endpoint
    async fn ready(&self) -> ApplicationStatus;
}

/// Variants to respond to health check request
#[derive(Debug, Clone, Copy)]
pub enum ApplicationStatus {
    /// returns 200 StatusCode
    UP,
    /// returns 501 StatusCode
    DOWN,
}

impl IntoResponse for ApplicationStatus {
    fn into_response(self) -> Response {
        match self {
            ApplicationStatus::UP => (StatusCode::OK, "UP").into_response(),
            ApplicationStatus::DOWN => (StatusCode::SERVICE_UNAVAILABLE, "DOWN").into_response(),
        }
    }
}

/// Default structure to mark application always alive and ready.
#[derive(Default, Debug, Clone)]
pub struct AlwaysReadyAndAlive {}

#[axum::async_trait]
impl Health for AlwaysReadyAndAlive {
    async fn alive(&self) -> ApplicationStatus {
        ApplicationStatus::UP
    }

    async fn ready(&self) -> ApplicationStatus {
        ApplicationStatus::UP
    }
}
