use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[axum::async_trait]
pub trait Health: Send + Sync + 'static {
    async fn alive(&self) -> ApplicationStatus;

    async fn ready(&self) -> ApplicationStatus;
}

#[derive(Debug, Clone, Copy)]
pub enum ApplicationStatus {
    UP,
    DOWN,
}

impl IntoResponse for ApplicationStatus {
    fn into_response(self) -> Response {
        match self {
            ApplicationStatus::UP => (StatusCode::OK, "UP").into_response(),
            ApplicationStatus::DOWN => (StatusCode::INTERNAL_SERVER_ERROR, "DOWN").into_response(),
        }
    }
}

#[derive(Default)]
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

#[derive(Default)]
pub struct NoHealth {}

#[axum::async_trait]
impl Health for NoHealth {
    async fn alive(&self) -> ApplicationStatus {
        ApplicationStatus::DOWN
    }

    async fn ready(&self) -> ApplicationStatus {
        ApplicationStatus::DOWN
    }
}
