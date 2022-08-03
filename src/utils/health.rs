use serde::Serialize;
use std::fmt;
use std::fmt::{Display, Formatter};

#[axum::async_trait]
pub trait Health: Send + Sync + 'static {
    async fn check(&self) -> HealthStatus;
}

#[derive(Serialize, Debug, Clone, Copy)]
pub enum HealthStatus {
    UP,
    DOWN,
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::DOWN
    }
}

impl Display for HealthStatus {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Default)]
pub struct AlwaysHealthy {}

#[axum::async_trait]
impl Health for AlwaysHealthy {
    async fn check(&self) -> HealthStatus {
        HealthStatus::UP
    }
}

#[derive(Default)]
pub struct NoHealth {}

#[axum::async_trait]
impl Health for NoHealth {
    async fn check(&self) -> HealthStatus {
        HealthStatus::DOWN
    }
}
